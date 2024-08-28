#![allow(clippy::inline_always)]

use std::cell::Ref;

use mlua::prelude::*;

use super::NativeDataHandle;

// Handle native data, provide type conversion between luavalue and native types
pub trait NativeConvert {
    // Convert luavalue into data, then write into ptr
    unsafe fn luavalue_into<'lua>(
        &self,
        lua: &'lua Lua,
        // type_userdata: &LuaAnyUserData<'lua>,
        offset: isize,
        data_handle: &Ref<dyn NativeDataHandle>,
        value: LuaValue<'lua>,
    ) -> LuaResult<()>;

    // Read data from ptr, then convert into luavalue
    unsafe fn luavalue_from<'lua>(
        &self,
        lua: &'lua Lua,
        // type_userdata: &LuaAnyUserData<'lua>,
        offset: isize,
        data_handle: &Ref<dyn NativeDataHandle>,
    ) -> LuaResult<LuaValue<'lua>>;
}
