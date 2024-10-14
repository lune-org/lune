use core::ffi::c_void;
use std::cell::Ref;
// use std::ptr;

use libffi::{
    // low::{closure_alloc, ffi_cif, CodePtr, RawCallback},
    low::{ffi_cif, CodePtr},
    // middle::Cif,
    // raw::ffi_prep_closure_loc,
};
use mlua::prelude::*;

use super::{GetNativeData, NativeArgInfo, NativeData, NativeResultInfo};

pub struct FfiCallable {
    cif: *mut ffi_cif,
    arg_info: *const Vec<NativeArgInfo>,
    result_info: *const NativeResultInfo,
    code: CodePtr,

    // Caching for better performance
    result_size: usize,
}

impl FfiCallable {
    pub unsafe fn new(
        cif: *mut ffi_cif,
        arg_info: *const Vec<NativeArgInfo>,
        result_info: *const NativeResultInfo,
        function_pointer: *const (),
    ) -> Self {
        let result_size = (*(*result_info).conv).get_size();
        Self {
            cif,
            arg_info,
            result_info,
            code: CodePtr::from_ptr(function_pointer.cast::<c_void>()),

            result_size,
        }
    }

    pub unsafe fn call(&self, result: &Ref<dyn NativeData>, args: LuaMultiValue) -> LuaResult<()> {
        result
            .check_boundary(0, self.result_size)
            .then_some(())
            .ok_or_else(|| LuaError::external("result boundary check failed"))
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
    }
}
