use core::ffi::c_void;
use std::ptr;

use libffi::{
    low::{ffi_cif, CodePtr},
    raw::ffi_call,
};
use mlua::prelude::*;

use super::GetFfiData;
use crate::ffi::{FfiArg, FfiResult};

pub struct CallableData {
    cif: *mut ffi_cif,
    arg_info_list: Vec<FfiArg>,
    result_info: FfiResult,
    code: CodePtr,
}

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

    // TODO? async call: if have no lua closure in arguments, fficallble can be called with async way

    pub unsafe fn call(&self, result: LuaValue, args: LuaMultiValue) -> LuaResult<()> {
        // cache Vec => unable to create async call but no allocation
        let mut arg_list = Vec::<*mut c_void>::with_capacity(self.arg_info_list.len());

        let result_pointer = if self.result_info.size == 0 {
            ptr::null_mut()
        } else {
            let result_data = result.get_ffi_data()?;
            if result_data.check_boundary(0, self.result_info.size) {
                return Err(LuaError::external("Result boundary check failed"));
            }
            result_data.get_pointer()
        }
        .cast::<c_void>();

        for index in 0..self.arg_info_list.len() {
            let arg_info = self.arg_info_list.get(index).unwrap();
            let arg = args
                .get(index)
                .ok_or_else(|| LuaError::external(format!("argument {index} required")))?;

            let arg_pointer = if let LuaValue::UserData(userdata) = arg {
                // BoxData, RefData, ...
                let data_handle = userdata.get_ffi_data()?;
                if !data_handle.check_boundary(0, arg_info.size) {
                    return Err(LuaError::external(format!(
                        "argument {index} boundary check failed"
                    )));
                }
                data_handle.get_pointer()
            } else {
                // FIXME: buffer, string here
                return Err(LuaError::external("unimpl"));
            };
            arg_list.push(arg_pointer.cast::<c_void>());
        }

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
        methods.add_method(
            "call",
            |_lua, this: &CallableData, mut args: LuaMultiValue| {
                let result = args.pop_front().ok_or_else(|| {
                    LuaError::external("First argument must be result data handle or nil")
                })?;
                // FIXME: clone
                unsafe { this.call(result, args) }
            },
        );
        // ref, leak ..?
    }
}
