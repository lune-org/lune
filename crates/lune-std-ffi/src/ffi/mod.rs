use std::cell::Ref;

use mlua::prelude::*;

pub mod association;
pub mod bit_mask;
mod cast;
pub mod libffi_helper;

pub use self::cast::num_cast;

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

    unsafe fn copy_data(
        &self,
        lua: &Lua,
        dst_offset: isize,
        src_offset: isize,
        dst: &Ref<dyn FfiData>,
        src: &Ref<dyn FfiData>,
    ) -> LuaResult<()>;

    unsafe fn stringify_data(
        &self,
        _lua: &Lua,
        _offset: isize,
        _data_handle: &Ref<dyn FfiData>,
    ) -> LuaResult<String> {
        Err(LuaError::external("Stringify method not implemented"))
    }
}

pub trait FfiData {
    fn check_inner_boundary(&self, offset: isize, size: usize) -> bool;
    unsafe fn get_inner_pointer(&self) -> *mut ();
    fn is_writable(&self) -> bool;
    fn is_readable(&self) -> bool;
    unsafe fn copy_from(
        &self,
        src: &Ref<dyn FfiData>,
        length: usize,
        dst_offset: isize,
        src_offset: isize,
    ) {
        self.get_inner_pointer()
            .byte_offset(dst_offset)
            .copy_from(src.get_inner_pointer().byte_offset(src_offset), length);
    }
}

pub struct FfiArg {
    pub size: usize,
    pub callback_ref_flag: u8,
}

impl Clone for FfiArg {
    fn clone(&self) -> Self {
        Self {
            size: self.size,
            callback_ref_flag: self.callback_ref_flag,
        }
    }
}

pub struct FfiResult {
    pub size: usize,
}

impl Clone for FfiResult {
    fn clone(&self) -> Self {
        Self { size: self.size }
    }
}
