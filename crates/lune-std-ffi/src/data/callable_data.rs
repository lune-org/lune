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

use super::{GetFfiData, RefData};
use crate::ffi::{FfiArg, FfiData, FfiResult};

pub struct CallableData {
    cif: *mut ffi_cif,
    arg_info_list: Vec<FfiArg>,
    result_info: FfiResult,
    code: CodePtr,
}

const VOID_RESULT_PTR: *mut () = ptr::null_mut();
const ZERO_SIZE_ARG_PTR: *mut *mut c_void = ptr::null_mut();

// Optimization:
// Use known size array in stack instead of creating new Vec to eliminate heap allocation
macro_rules! create_caller {
    ($len:expr) => {
        |callable: &CallableData, result: LuaValue, args: LuaMultiValue| unsafe {
            // Get `rvalue: *mut c_void` result pointer
            let result_pointer = if callable.result_info.size == 0 {
                VOID_RESULT_PTR
            } else {
                result.get_ffi_data()?.get_inner_pointer()
            }
            .cast::<c_void>();

            // Create `avalue: *mut *mut c_void` argument list
            let mut arg_list: [MaybeUninit<*mut c_void>; $len] = [MaybeUninit::uninit(); $len];
            for (index, arg) in arg_list.iter_mut().enumerate() {
                let arg_value = args
                    .get(index)
                    .ok_or_else(|| LuaError::external(format!("Argument {index} required")))?
                    .as_userdata()
                    .ok_or_else(|| LuaError::external("Argument must be a RefData"))?;

                if let Ok(arg_ref) = arg_value.borrow::<RefData>() {
                    arg.write(arg_ref.get_inner_pointer().cast::<c_void>());
                } else {
                    return Err(LuaError::external("Argument must be a RefData"));
                }
            }

            ffi_call(
                callable.cif,
                Some(*callable.code.as_safe_fun()),
                result_pointer,
                // SAFETY: MaybeUninit<T> has the same layout as `T`, and initialized above
                mem::transmute::<[MaybeUninit<*mut c_void>; $len], [*mut c_void; $len]>(arg_list)
                    .as_mut_ptr(),
            );

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
    let result_pointer = if callable.result_info.size == 0 {
        VOID_RESULT_PTR
    } else {
        result.get_ffi_data()?.get_inner_pointer()
    }
    .cast::<c_void>();

    ffi_call(
        callable.cif,
        Some(*callable.code.as_safe_fun()),
        result_pointer,
        ZERO_SIZE_ARG_PTR,
    );

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

    pub unsafe fn call(&self, result: LuaValue, args: LuaMultiValue) -> LuaResult<()> {
        let arg_len = self.arg_info_list.len();
        // Optimization: use sized caller when possible
        if arg_len < SIZED_CALLERS.len() {
            return SIZED_CALLERS[arg_len](self, result, args);
        }

        // Get `rvalue: *mut c_void` result pointer
        let result_pointer = if self.result_info.size == 0 {
            VOID_RESULT_PTR
        } else {
            result.get_ffi_data()?.get_inner_pointer()
        }
        .cast::<c_void>();

        // Create `avalue: *mut *mut c_void` argument list
        let mut arg_list = Vec::<*mut c_void>::with_capacity(arg_len);
        for index in 0..arg_len {
            let arg_value = args
                .get(index)
                .ok_or_else(|| LuaError::external(format!("Argument {index} required")))?
                .as_userdata()
                .ok_or_else(|| LuaError::external("Argument must be a RefData"))?;

            if let Ok(arg_ref) = arg_value.borrow::<RefData>() {
                arg_list.push(arg_ref.get_inner_pointer().cast::<c_void>());
            } else {
                return Err(LuaError::external("Argument must be a RefData"));
            }
        }

        // Call libffi::raw::ffi_call
        ffi_call(
            self.cif,
            Some(*self.code.as_safe_fun()),
            result_pointer,
            arg_list.as_mut_ptr(),
        );

        Ok(())
    }
}

impl LuaUserData for CallableData {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(
            LuaMetaMethod::Call,
            |_lua, this: &CallableData, mut args: LuaMultiValue| {
                let result = args.pop_front().ok_or_else(|| {
                    LuaError::external("First argument 'result' must be a RefData, BoxData or nil")
                })?;
                unsafe { this.call(result, args) }
            },
        );
    }
}
