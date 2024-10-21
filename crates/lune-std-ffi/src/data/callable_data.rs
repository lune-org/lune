use core::ffi::c_void;
use std::ptr;

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
        let mut arg_list = Vec::<*mut c_void>::with_capacity(self.arg_info_list.len());

        let result_pointer = if self.result_info.size == 0 {
            ptr::null_mut()
        } else {
            let result_data = result.get_ffi_data()?;
            if !result_data.check_inner_boundary(0, self.result_info.size) {
                return Err(LuaError::external("Result boundary check failed"));
            }
            result_data.get_inner_pointer()
        }
        .cast::<c_void>();

        for index in 0..self.arg_info_list.len() {
            let arg_value = args
                .get(index)
                .ok_or_else(|| LuaError::external(format!("argument {index} required")))?
                .as_userdata()
                .ok_or_else(|| LuaError::external("argument should be Ref"))?;

            let arg_ref = arg_value.borrow::<RefData>()?;

            arg_list.push(arg_ref.get_inner_pointer().cast::<c_void>());
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
        methods.add_meta_method(
            LuaMetaMethod::Call,
            |_lua, this: &CallableData, mut args: LuaMultiValue| {
                let result = args.pop_front().ok_or_else(|| {
                    LuaError::external("First argument must be result data handle or nil")
                })?;
                unsafe { this.call(result, args) }
            },
        );
    }
}
