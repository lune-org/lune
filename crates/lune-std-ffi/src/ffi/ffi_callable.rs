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

use super::{
    bit_mask::u8_test_not, ffi_native::NativeArgInfo, FfiRef, FfiRefFlag, GetNativeData,
    NativeConvert, NativeData,
};

// unsafe extern "C" fn callback() {
//     _cif: ffi_cif,
//     result: &mut
// }

// pub type RawCallback = unsafe extern "C" fn(cif: *mut ffi_cif, result: *mut c_void, args: *mut *mut c_void, userdata: *mut c_void);
// pub unsafe extern "C" fn ffi_prep_raw_closure(
//     arg1: *mut ffi_raw_closure,
//     cif: *mut ffi_cif,
//     fun: Option<unsafe extern "C" fn(_: *mut ffi_cif, _: *mut c_void, _: *mut ffi_raw, _: *mut c_void)>,
//     user_data: *mut c_void
// ) -> u32

// pub fn ffi_prep_raw_closure_loc(
//     arg1: *mut ffi_raw_closure,
//     cif: *mut ffi_cif,
//     fun: Option<
//         unsafe extern "C" fn(
//             arg1: *mut ffi_cif,
//             arg2: *mut c_void,
//             arg3: *mut ffi_raw,
//             arg4: *mut c_void,
//         ),
//     >,
//     user_data: *mut c_void,
//     codeloc: *mut c_void,
// ) -> ffi_status;

pub struct FfiCallable {
    cif: *mut ffi_cif,
    arg_type_list: Vec<NativeArgInfo>,
    result_size: usize,
    code: CodePtr,
}

impl FfiCallable {
    pub unsafe fn new(
        cif: *mut ffi_cif,
        arg_type_list: Vec<NativeArgInfo>,
        result_size: usize,
        function_ref: FfiRef,
    ) -> LuaResult<Self> {
        if u8_test_not(function_ref.flags, FfiRefFlag::Function.value()) {
            return Err(LuaError::external("ref is not function pointer"));
        }
        Ok(Self {
            cif,
            arg_type_list,
            result_size,
            code: CodePtr::from_ptr(function_ref.get_pointer(0).cast::<c_void>()),
        })
    }

    pub unsafe fn call(&self, result: &Ref<dyn NativeData>, args: LuaMultiValue) -> LuaResult<()> {
        result
            .check_boundary(0, self.result_size)
            .then_some(())
            .ok_or_else(|| LuaError::external("result boundary check failed"))
    }
    // pub fn new_from_lua_table(lua: &Lua, args: LuaTable, ret: LuaAnyUserData) -> LuaResult<Self> {
    //     let args_types = libffi_type_list_from_table(lua, &args)?;
    //     let ret_type = libffi_type_from_userdata(lua, &ret)?;

    //     Ok(Self::new(
    //         args_types,
    //         ret_type,
    //         unsafe { get_conv_list_from_table(&args)? },
    //         unsafe { get_conv(&ret)? },
    //     ))
    // }
    // pub fn call() {

    // }
}

impl LuaUserData for FfiCallable {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method(
            "call",
            |_lua, this: &FfiCallable, mut args: LuaMultiValue| {
                let LuaValue::UserData(result) = args.pop_front().ok_or_else(|| {
                    LuaError::external("first argument must be result data handle")
                })?
                else {
                    return Err(LuaError::external(""));
                };
                let call_result = unsafe { this.call(&result.get_data_handle()?, args) };
                call_result
            },
        )
    }
}
