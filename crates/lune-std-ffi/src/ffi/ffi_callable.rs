use core::ffi::c_void;
use std::cell::Ref;

use libffi::{
    low::{ffi_cif, CodePtr},
    raw::ffi_call,
};
use mlua::prelude::*;

use super::{GetNativeData, NativeArgInfo, NativeData, NativeResultInfo};

pub struct FfiCallable {
    cif: *mut ffi_cif,
    arg_info_list: *const Vec<NativeArgInfo>,
    result_info: *const NativeResultInfo,
    code: CodePtr,
}

impl FfiCallable {
    pub unsafe fn new(
        cif: *mut ffi_cif,
        arg_info_list: *const Vec<NativeArgInfo>,
        result_info: *const NativeResultInfo,
        function_pointer: *const (),
    ) -> Self {
        Self {
            cif,
            arg_info_list,
            result_info,
            code: CodePtr::from_ptr(function_pointer.cast::<c_void>()),
        }
    }

    pub unsafe fn call(&self, result: &Ref<dyn NativeData>, args: LuaMultiValue) -> LuaResult<()> {
        result
            .check_boundary(0, self.result_info.as_ref().unwrap().size)
            .then_some(())
            .ok_or_else(|| LuaError::external("result boundary check failed"))?;

        // cache Vec => unable to create async call but no allocation
        let arg_info_list = self.arg_info_list.as_ref().unwrap();
        let mut arg_list = Vec::<*mut c_void>::with_capacity(arg_info_list.len());

        for index in 0..arg_info_list.len() {
            let arg_info = arg_info_list.get(index).unwrap();
            let arg = args
                .get(index)
                .ok_or_else(|| LuaError::external(format!("argument {index} required")))?;
            let arg_pointer = if let LuaValue::UserData(userdata) = arg {
                let data_handle = userdata.get_data_handle()?;
                data_handle
                    .check_boundary(0, arg_info.size)
                    .then_some(())
                    .ok_or_else(|| {
                        LuaError::external(format!("argument {index} boundary check failed"))
                    })?;
                data_handle.get_pointer(0)
            } else {
                return Err(LuaError::external("unimpl"));
            };
            arg_list.push(arg_pointer.cast::<c_void>());
        }

        ffi_call(
            self.cif,
            Some(*self.code.as_safe_fun()),
            result.get_pointer(0).cast::<c_void>(),
            arg_list.as_mut_ptr(),
        );

        Ok(())
    }
}

impl LuaUserData for FfiCallable {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method(
            "call",
            |_lua, this: &FfiCallable, mut args: LuaMultiValue| {
                let result_userdata = args.pop_front().ok_or_else(|| {
                    LuaError::external("first argument must be result data handle")
                })?;
                let LuaValue::UserData(result) = result_userdata else {
                    return Err(LuaError::external(""));
                };
                // FIXME: clone
                unsafe { this.call(&result.clone().get_data_handle()?, args) }
            },
        );
        // ref, leak ..?
    }
}
