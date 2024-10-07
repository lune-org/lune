use std::cell::Ref;

use mlua::prelude::*;

use super::NativeDataHandle;

// Handle native data, provide type conversion between luavalue and native types
pub trait NativeCall {
    // Call native function
    unsafe fn call_native(
        &self,
        lua: &Lua,
        arg: LuaMultiValue,
        ret: &Ref<dyn NativeDataHandle>,
    ) -> LuaResult<()>;

    // Call lua closure
    unsafe fn call_lua(&self, lua: &Lua, arg: LuaMultiValue, ret: *mut ()) -> LuaResult<()>;
}
