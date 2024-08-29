#![allow(clippy::inline_always)]

use mlua::prelude::*;

use super::ReadWriteHandle;

// Handle native data, provide type conversion between luavalue and native types
pub trait NativeConvert {
    // Convert luavalue into data, then write into ptr
    fn luavalue_into<'lua>(
        &self,
        into: impl ReadWriteHandle,
        lua: &'lua Lua,
        value: LuaValue<'lua>,
    ) -> LuaResult<()>;

    // Read data from ptr, then convert into luavalue
    fn luavalue_from<'lua>(
        &self,
        from: impl ReadWriteHandle,
        lua: &'lua Lua,
    ) -> LuaResult<LuaValue<'lua>>;
}
