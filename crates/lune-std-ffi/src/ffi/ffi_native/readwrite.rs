use std::cell::Ref;

use lune_utils::fmt::{pretty_format_value, ValueFormatConfig};
use mlua::prelude::*;

use super::super::{FfiBox, FfiRef};

pub trait ReadWriteHandle {
    fn check_boundary(&self, offset: isize, size: usize) -> bool;
    fn check_readable(&self, userdata: &LuaAnyUserData, offset: isize, size: usize) -> bool;
    unsafe fn get_pointer(&self, offset: isize) -> *mut ();
}

pub trait GetReadWriteHandle {
    fn get_data_handle<'a>(&'a self) -> LuaResult<Ref<'a, dyn ReadWriteHandle>>;
}
impl GetReadWriteHandle for LuaAnyUserData<'_> {
    fn get_data_handle<'a>(&'a self) -> LuaResult<Ref<'a, dyn ReadWriteHandle>> {
        if self.is::<FfiBox>() {
            Ok(self.borrow::<FfiBox>()? as Ref<dyn ReadWriteHandle>)
        } else if self.is::<FfiRef>() {
            Ok(self.borrow::<FfiRef>()? as Ref<dyn ReadWriteHandle>)
        // } else if self.is::<FfiRaw>() {
        //     Ok(self.borrow::<FfiRaw>()? as Ref<dyn ReadWriteHandle>)
        } else {
            let config = ValueFormatConfig::new();
            Err(LuaError::external(format!(
                "Expected FfiBox, FfiRef or FfiRaw. got {}",
                pretty_format_value(&LuaValue::UserData(self.to_owned()), &config)
            )))
        }
    }
}
