use core::ffi::c_void;
use std::{borrow::Borrow, ptr};

use libffi::{
    low::{closure_alloc, closure_free, ffi_cif},
    raw::{ffi_closure, ffi_prep_closure_loc},
};
use mlua::prelude::*;

use super::{
    association_names::REF_INNER,
    ref_data::{RefBounds, RefData, RefFlag, UNSIZED_BOUNDS},
};
use crate::ffi::{
    association,
    libffi_helper::{ffi_status_assert, SIZE_OF_POINTER},
    FfiArg, FfiData, FfiResult,
};

pub struct ClosureData {
    lua: *const Lua,
    closure: *mut ffi_closure,
    code: Box<*mut c_void>,
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

const RESULT_REF_FLAGS: u8 = RefFlag::Writable.value() | RefFlag::Offsetable.value();
const CLOSURE_REF_FLAGS: u8 = RefFlag::Function.value();

// Process C -> Lua function call
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
        .call::<_, ()>(LuaMultiValue::from_vec(args))
        .unwrap();
}

impl ClosureData {
    // Allocate new ffi closure with lua function
    pub fn alloc(
        lua: &Lua,
        cif: *mut ffi_cif,
        arg_info_list: Vec<FfiArg>,
        result_info: FfiResult,
        func: LuaRegistryKey,
    ) -> LuaResult<LuaAnyUserData> {
        let (closure, code) = closure_alloc();
        let code = code.as_mut_ptr();

        let closure_data = lua.create_userdata(ClosureData {
            lua: ptr::from_ref(lua),
            closure,
            code: Box::new(code),
            arg_info_list,
            result_info,
            func,
        })?;

        let closure_data_ptr = ptr::from_ref(&*closure_data.borrow::<ClosureData>()?);

        ffi_status_assert(unsafe {
            ffi_prep_closure_loc(
                closure,
                cif,
                Some(callback),
                closure_data_ptr.cast::<c_void>().cast_mut(),
                code,
            )
        })?;

        Ok(closure_data)
    }
}

impl FfiData for ClosureData {
    unsafe fn get_inner_pointer(&self) -> *mut () {
        ptr::from_ref::<*mut c_void>(&*self.code)
            .cast::<()>()
            .cast_mut()
    }
    fn check_inner_boundary(&self, offset: isize, size: usize) -> bool {
        (offset as usize) + size <= SIZE_OF_POINTER
    }
    fn is_readable(&self) -> bool {
        false
    }
    fn is_writable(&self) -> bool {
        false
    }
}

impl LuaUserData for ClosureData {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("ref", |lua, this: LuaAnyUserData| {
            let ref_data = lua.create_userdata(RefData::new(
                unsafe { this.borrow::<ClosureData>()?.get_inner_pointer() },
                CLOSURE_REF_FLAGS,
                UNSIZED_BOUNDS,
            ))?;
            association::set(lua, REF_INNER, &ref_data, &this)?;
            Ok(ref_data)
        });
    }
}
