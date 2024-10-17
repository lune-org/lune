use std::cell::Ref;

use mlua::prelude::*;

mod arg;
pub mod association;
pub mod bit_mask;
mod cast;
pub mod libffi_helper;
mod result;

pub trait FfiSize {
    fn get_size(&self) -> usize;
}

pub trait FfiSignedness {
    fn get_signedness(&self) -> bool {
        false
    }
}

// Provide type conversion between luavalue and ffidata types
pub trait FfiConvert {
    // Write LuaValue into FfiData
    unsafe fn value_into_data<'lua>(
        &self,
        lua: &'lua Lua,
        offset: isize,
        data_handle: &Ref<dyn FfiData>,
        value: LuaValue<'lua>,
    ) -> LuaResult<()>;

    // Read LuaValue from FfiData
    unsafe fn value_from_data<'lua>(
        &self,
        lua: &'lua Lua,
        offset: isize,
        data_handle: &Ref<dyn FfiData>,
    ) -> LuaResult<LuaValue<'lua>>;
}

pub trait FfiData {
    fn check_boundary(&self, offset: isize, size: usize) -> bool;
    unsafe fn get_pointer(&self) -> *mut ();
    fn is_writable(&self) -> bool;
    fn is_readable(&self) -> bool;
}

pub use self::{arg::FfiArgInfo, cast::num_cast, result::FfiResultInfo};
