use core::ffi::c_void;
use std::cell::Ref;

use libffi::{
    low::{ffi_cif, CodePtr},
    raw::ffi_call,
};
use mlua::prelude::*;

use super::{FfiData, GetFfiData};
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

    pub unsafe fn call(&self, result: &Ref<dyn FfiData>, args: LuaMultiValue) -> LuaResult<()> {
        result
            .check_boundary(0, self.result_info.size)
            .then_some(())
            .ok_or_else(|| LuaError::external("result boundary check failed"))?;

        // cache Vec => unable to create async call but no allocation
        let mut arg_list = Vec::<*mut c_void>::with_capacity(self.arg_info_list.len());

        for index in 0..self.arg_info_list.len() {
            let arg_info = self.arg_info_list.get(index).unwrap();
            let arg = args
                .get(index)
                .ok_or_else(|| LuaError::external(format!("argument {index} required")))?;
            let arg_pointer = if let LuaValue::UserData(userdata) = arg {
                let data_handle = userdata.get_ffi_data()?;
                data_handle
                    .check_boundary(0, arg_info.size)
                    .then_some(())
                    .ok_or_else(|| {
                        LuaError::external(format!("argument {index} boundary check failed"))
                    })?;
                data_handle.get_pointer()
            } else {
                return Err(LuaError::external("unimpl"));
            };
            arg_list.push(arg_pointer.cast::<c_void>());
        }

        ffi_call(
            self.cif,
            Some(*self.code.as_safe_fun()),
            result.get_pointer().cast::<c_void>(),
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
                let result_userdata = args.pop_front().ok_or_else(|| {
                    LuaError::external("first argument must be result data handle")
                })?;
                let LuaValue::UserData(result) = result_userdata else {
                    return Err(LuaError::external(""));
                };
                // FIXME: clone
                unsafe { this.call(&result.clone().get_ffi_data()?, args) }
            },
        );
        // ref, leak ..?
    }
}
