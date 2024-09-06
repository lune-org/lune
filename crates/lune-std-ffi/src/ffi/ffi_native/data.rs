use std::cell::Ref;

use lune_utils::fmt::{pretty_format_value, ValueFormatConfig};
use mlua::prelude::*;

use super::super::{FfiBox, FfiRef};

pub trait NativeData {
    fn check_boundary(&self, offset: isize, size: usize) -> bool;
    unsafe fn get_pointer(&self, offset: isize) -> *mut ();
}

pub trait GetNativeData {
    fn get_data_handle(&self) -> LuaResult<Ref<dyn NativeData>>;
}

// I tried to remove dyn (which have little bit costs)
// But, maybe this is best option for now.
// If remove dyn, we must spam self.is::<>() / self.borrow::<>()?
// more costly....
impl GetNativeData for LuaAnyUserData<'_> {
    fn get_data_handle(&self) -> LuaResult<Ref<dyn NativeData>> {
        if self.is::<FfiBox>() {
            Ok(self.borrow::<FfiBox>()? as Ref<dyn NativeData>)
        } else if self.is::<FfiRef>() {
            Ok(self.borrow::<FfiRef>()? as Ref<dyn NativeData>)
        // } else if self.is::<FfiRaw>() {
        // Ok(self.borrow::<FfiRaw>()? as Ref<dyn ReadWriteHandle>)
        } else {
            let config = ValueFormatConfig::new();
            Err(LuaError::external(format!(
                "Expected FfiBox, FfiRef or FfiRaw. got {}",
                // what?
                pretty_format_value(&LuaValue::UserData(self.to_owned()), &config)
            )))
        }
    }
}
