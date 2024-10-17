use core::ffi::c_void;
use std::{borrow::Borrow, ptr};

use libffi::{
    low::{closure_alloc, closure_free, ffi_cif, CodePtr},
    raw::{ffi_closure, ffi_prep_closure_loc},
};
use mlua::prelude::*;

use super::ref_data::{RefBounds, RefData, RefFlag};
use crate::ffi::{
    libffi_helper::{FFI_STATUS_NAMES, SIZE_OF_POINTER},
    FfiArg, FfiData, FfiResult,
};

pub struct ClosureData {
    lua: *const Lua,
    closure: *mut ffi_closure,
    code: CodePtr,
    arg_info_list: Vec<FfiArg>,
    result_info: FfiResult,
    func: LuaRegistryKey,
}

impl Drop for ClosureData {
    fn drop(&mut self) {
        unsafe {
            closure_free(self.closure);
        }
    }
}

const RESULT_REF_FLAGS: u8 =
    RefFlag::Leaked.value() | RefFlag::Writable.value() | RefFlag::Offsetable.value();

unsafe extern "C" fn callback(
    cif: *mut ffi_cif,
    result_pointer: *mut c_void,
    arg_pointers: *mut *mut c_void,
    closure_data: *mut c_void,
) {
    let closure_data = closure_data.cast::<ClosureData>().as_ref().unwrap();
    let lua = closure_data.lua.as_ref().unwrap();
    let len = (*cif).nargs as usize;
    let mut args = Vec::<LuaValue>::with_capacity(len + 1);

    // Push result pointer (ref)
    args.push(LuaValue::UserData(
        lua.create_userdata(RefData::new(
            result_pointer.cast::<()>(),
            RESULT_REF_FLAGS,
            RefBounds::new(0, closure_data.result_info.size),
        ))
        .unwrap(),
    ));

    // Push arg pointer (ref)
    for i in 0..len {
        let arg_info = closure_data.arg_info_list.get(i).unwrap();
        args.push(LuaValue::UserData(
            lua.create_userdata(RefData::new(
                (*arg_pointers.add(i)).cast::<()>(),
                arg_info.callback_ref_flag,
                RefBounds::new(0, arg_info.size),
            ))
            .unwrap(),
        ));
    }

    closure_data
        .func
        .borrow()
        .into_lua(lua)
        .unwrap()
        .as_function()
        .unwrap()
        .call::<_, ()>(args)
        .unwrap();
}

impl ClosureData {
    pub fn new(
        lua: *const Lua,
        cif: *mut ffi_cif,
        arg_info_list: Vec<FfiArg>,
        result_info: FfiResult,
        func: LuaRegistryKey,
    ) -> LuaResult<ClosureData> {
        let (closure, code) = closure_alloc();

        let closure_data = ClosureData {
            lua,
            closure,
            code,
            arg_info_list,
            result_info,
            func,
        };

        let prep_result = unsafe {
            ffi_prep_closure_loc(
                closure,
                cif,
                Some(callback),
                ptr::from_ref(&closure_data).cast::<c_void>().cast_mut(),
                code.as_mut_ptr(),
            )
        };

        if prep_result != 0 {
            Err(LuaError::external(format!(
                "ffi_get_struct_offsets failed. expected result {}, got {}",
                FFI_STATUS_NAMES[0], FFI_STATUS_NAMES[prep_result as usize]
            )))
        } else {
            Ok(closure_data)
        }
    }
}

impl FfiData for ClosureData {
    unsafe fn get_pointer(&self) -> *mut () {
        self.code.as_mut_ptr().cast::<()>()
    }
    fn check_boundary(&self, offset: isize, size: usize) -> bool {
        (offset as usize) + size <= SIZE_OF_POINTER
    }
    fn is_readable(&self) -> bool {
        false
    }
    fn is_writable(&self) -> bool {
        false
    }
}

impl LuaUserData for ClosureData {}
