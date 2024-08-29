use std::{cell::Ref, ops::Deref};

use lune_utils::fmt::{pretty_format_value, ValueFormatConfig};
use mlua::prelude::*;

use super::super::{FfiBox, FfiRef};

pub trait NativeDataHandle {
    fn check_boundary(&self, offset: isize, size: usize) -> bool;
    fn check_readable(&self, userdata: &LuaAnyUserData, offset: isize, size: usize) -> bool;
    fn checek_writable(&self, userdata: &LuaAnyUserData, offset: isize, size: usize) -> bool;
    unsafe fn get_pointer(&self, offset: isize) -> *mut ();
}

pub trait GetNativeDataHandle {
    fn get_data_handle<'a>(&'a self) -> LuaResult<Ref<'a, dyn NativeDataHandle>>;
}
impl GetNativeDataHandle for LuaAnyUserData<'_> {
    fn get_data_handle<'a>(&'a self) -> LuaResult<Ref<'a, dyn NativeDataHandle>> {
        if self.is::<FfiBox>() {
            Ok(self.borrow::<FfiBox>()? as Ref<dyn NativeDataHandle>)
        } else if self.is::<FfiRef>() {
            Ok(self.borrow::<FfiRef>()? as Ref<dyn NativeDataHandle>)
        // } else if self.is::<FfiRaw>() {
        //     Ok(self.borrow::<FfiRaw>()? as Ref<dyn ReadWriteHandle>)
        } else {
            let config = ValueFormatConfig::new();
            Err(LuaError::external(format!(
                "Expected FfiBox, FfiRef or FfiRaw. got {}",
                // what?
                pretty_format_value(&LuaValue::UserData(self.deref().to_owned()), &config)
            )))
        }
    }
}
