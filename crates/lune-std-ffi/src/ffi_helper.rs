use std::ffi::c_void;

use mlua::prelude::*;

use crate::ffi_box::FfiBox;
use crate::ffi_ref::FfiRef;

pub unsafe fn get_ptr_from_userdata(
    userdata: &LuaAnyUserData,
    offset: Option<isize>,
) -> LuaResult<*mut c_void> {
    let ptr = if userdata.is::<FfiBox>() {
        userdata.borrow::<FfiBox>()?.get_ptr()
    } else if userdata.is::<FfiRef>() {
        userdata.borrow::<FfiRef>()?.get_ptr()
    } else {
        return Err(LuaError::external("asdf"));
    };

    let ptr = if let Some(t) = offset {
        ptr.offset(t)
    } else {
        ptr
    };

    Ok(ptr)
}
