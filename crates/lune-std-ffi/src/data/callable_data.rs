use core::ffi::c_void;
use std::{
    mem::{self, MaybeUninit},
    ptr,
};

use libffi::{
    low::{ffi_cif, CodePtr},
    raw::ffi_call,
};
use mlua::prelude::*;

use super::{FfiDataRef, GetFfiData};
use crate::ffi::{libffi_helper::SIZE_OF_FFI_ARG, FfiArg, FfiData, FfiResult};

// A function pointer that luau can call. it stores libffi cif for calling convention.
pub struct CallableData {
    cif: *mut ffi_cif,
    arg_info_list: Vec<FfiArg>,
    result_info: FfiResult,
    code: CodePtr,
}

const VOID_RESULT_PTR: *mut c_void = ptr::null_mut();
const ZERO_SIZE_ARG_PTR: *mut *mut c_void = ptr::null_mut();

// Get the pointer libffi reads an argument from. The userdata stays alive via
// the caller's LuaMultiValue, so the pointer is valid for the call.
#[inline]
unsafe fn get_arg_pointer(
    arg_info: &FfiArg,
    arg_value: Option<&LuaValue>,
    index: usize,
) -> LuaResult<*mut c_void> {
    let arg_value = arg_value
        .ok_or_else(|| LuaError::external(format!("Argument {index} is required, but got none")))?;

    let data = arg_value.get_ffi_data().map_err(|_| {
        LuaError::external(format!(
            "Argument {index} must be a BoxData, RefData or ClosureData"
        ))
    })?;

    let pointer = data.get_inner_pointer();
    if pointer.is_null() {
        return Err(LuaError::external(format!(
            "Argument {index} points to a null address"
        )));
    }
    if !data.check_inner_boundary(0, arg_info.size) {
        return Err(LuaError::external(format!(
            "Argument {index} is too small for its type (expected {} bytes)",
            arg_info.size
        )));
    }

    Ok(pointer.cast::<c_void>())
}

// Where libffi writes the return value, plus what's needed to copy it back out
// when it was widened.
struct ResultTarget {
    // Kept alive so the destination pointer stays valid during the call
    data: Option<FfiDataRef>,
    bounce: bool,
}

// Optimization:
// Use known size array in stack instead of creating new Vec to eliminate heap allocation
macro_rules! create_caller {
    ($len:expr) => {
        |callable: &CallableData, result: LuaValue, args: LuaMultiValue| unsafe {
            let mut widen_buffer = [0u8; SIZE_OF_FFI_ARG];
            let (result_target, result_pointer) =
                callable.prepare_result(result, &mut widen_buffer)?;

            // Create `avalue: *mut *mut c_void` argument list
            let mut arg_list: [MaybeUninit<*mut c_void>; $len] = [MaybeUninit::uninit(); $len];
            for (index, arg) in arg_list.iter_mut().enumerate() {
                arg.write(get_arg_pointer(
                    &callable.arg_info_list[index],
                    args.get(index),
                    index,
                )?);
            }

            ffi_call(
                callable.cif,
                Some(*callable.code.as_safe_fun()),
                result_pointer,
                // SAFETY: MaybeUninit<T> has the same layout as `T`, and initialized above
                mem::transmute::<[MaybeUninit<*mut c_void>; $len], [*mut c_void; $len]>(arg_list)
                    .as_mut_ptr(),
            );

            callable.finish_result(&result_target, &widen_buffer);
            Ok(())
        }
    };
}

// Optimization:
// Call without arguments
unsafe fn zero_size_caller(
    callable: &CallableData,
    result: LuaValue,
    _args: LuaMultiValue,
) -> LuaResult<()> {
    let mut widen_buffer = [0u8; SIZE_OF_FFI_ARG];
    let (result_target, result_pointer) = callable.prepare_result(result, &mut widen_buffer)?;

    ffi_call(
        callable.cif,
        Some(*callable.code.as_safe_fun()),
        result_pointer,
        ZERO_SIZE_ARG_PTR,
    );

    callable.finish_result(&result_target, &widen_buffer);
    Ok(())
}

// Optimization: sized callers
type Caller =
    unsafe fn(callable: &CallableData, result: LuaValue, args: LuaMultiValue) -> LuaResult<()>;
const SIZED_CALLERS: [Caller; 13] = [
    zero_size_caller,
    create_caller!(1),
    create_caller!(2),
    create_caller!(3),
    create_caller!(4),
    create_caller!(5),
    create_caller!(6),
    create_caller!(7),
    create_caller!(8),
    create_caller!(9),
    create_caller!(10),
    create_caller!(11),
    create_caller!(12),
];

impl CallableData {
    pub unsafe fn new(
        cif: *mut ffi_cif,
        arg_info_list: Vec<FfiArg>,
        result_info: FfiResult,
        function_pointer: *const (),
    ) -> Self {
        Self {
            cif,
            arg_info_list,
            result_info,
            code: CodePtr::from_ptr(function_pointer.cast::<c_void>()),
        }
    }

    // Stringify for pretty-print, with hex format address
    pub fn stringify(&self) -> String {
        format!("0x{:x}", self.code.as_ptr() as usize)
    }

    // libffi widens integral returns narrower than ffi_arg to a full word, so
    // small returns bounce through widen_buffer and get copied out by
    // finish_result; larger returns are written straight to the destination.
    unsafe fn prepare_result(
        &self,
        result: LuaValue,
        widen_buffer: &mut [u8; SIZE_OF_FFI_ARG],
    ) -> LuaResult<(ResultTarget, *mut c_void)> {
        let size = self.result_info.size;
        if size == 0 {
            return Ok((
                ResultTarget {
                    data: None,
                    bounce: false,
                },
                VOID_RESULT_PTR,
            ));
        }

        let data = result.get_ffi_data().map_err(|_| {
            LuaError::external(
                "Result must be a BoxData or RefData when the function returns a value",
            )
        })?;
        if !data.is_writable() {
            return Err(LuaError::external("Result is not writable"));
        }
        if !data.check_inner_boundary(0, size) {
            return Err(LuaError::external(format!(
                "Result is too small for the return type (expected {size} bytes)"
            )));
        }

        if size < SIZE_OF_FFI_ARG {
            Ok((
                ResultTarget {
                    data: Some(data),
                    bounce: true,
                },
                widen_buffer.as_mut_ptr().cast::<c_void>(),
            ))
        } else {
            let pointer = data.get_inner_pointer().cast::<c_void>();
            Ok((
                ResultTarget {
                    data: Some(data),
                    bounce: false,
                },
                pointer,
            ))
        }
    }

    // Copy a widened return value out of the bounce buffer into the destination.
    unsafe fn finish_result(&self, target: &ResultTarget, widen_buffer: &[u8; SIZE_OF_FFI_ARG]) {
        if !target.bounce {
            return;
        }
        let size = self.result_info.size;
        let destination = target
            .data
            .as_ref()
            .expect("bounced result always has a destination")
            .get_inner_pointer()
            .cast::<u8>();

        // On little-endian the value occupies the low `size` bytes; on
        // big-endian, integral returns are right-justified within the word.
        let source_offset = if cfg!(target_endian = "big") {
            SIZE_OF_FFI_ARG - size
        } else {
            0
        };
        destination.copy_from(widen_buffer.as_ptr().add(source_offset), size);
    }

    pub unsafe fn call(&self, result: LuaValue, args: LuaMultiValue) -> LuaResult<()> {
        let arg_len = self.arg_info_list.len();
        // Optimization: use sized caller when possible
        if arg_len < SIZED_CALLERS.len() {
            return SIZED_CALLERS[arg_len](self, result, args);
        }

        let mut widen_buffer = [0u8; SIZE_OF_FFI_ARG];
        let (result_target, result_pointer) = self.prepare_result(result, &mut widen_buffer)?;

        // Create `avalue: *mut *mut c_void` argument list
        let mut arg_list = Vec::<*mut c_void>::with_capacity(arg_len);
        for index in 0..arg_len {
            arg_list.push(get_arg_pointer(
                &self.arg_info_list[index],
                args.get(index),
                index,
            )?);
        }

        // Call libffi::raw::ffi_call
        ffi_call(
            self.cif,
            Some(*self.code.as_safe_fun()),
            result_pointer,
            arg_list.as_mut_ptr(),
        );

        self.finish_result(&result_target, &widen_buffer);
        Ok(())
    }
}

impl LuaUserData for CallableData {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(
            LuaMetaMethod::Call,
            |_lua, this: &CallableData, mut args: LuaMultiValue| {
                let result = args.pop_front().ok_or_else(|| {
                    LuaError::external("First argument 'result' must be a RefData, BoxData or nil")
                })?;
                unsafe { this.call(result, args) }
            },
        );
        methods.add_meta_method(LuaMetaMethod::ToString, |_lua, this, ()| {
            Ok(this.stringify())
        });
    }
}
