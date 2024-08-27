#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;

use super::super::ffi_helper::get_ptr_from_userdata;

// Handle native data, provide type conversion between luavalue and native types
pub trait NativeConvert {
    // Convert luavalue into data, then write into ptr
    fn luavalue_into_ptr<'lua>(
        &self,
        this: &LuaAnyUserData<'lua>,
        lua: &'lua Lua,
        value: LuaValue<'lua>,
        ptr: *mut (),
    ) -> LuaResult<()>;

    // Read data from ptr, then convert into luavalue
    fn ptr_into_luavalue<'lua>(
        &self,
        this: &LuaAnyUserData<'lua>,
        lua: &'lua Lua,
        ptr: *mut (),
    ) -> LuaResult<LuaValue<'lua>>;

    // Read data from userdata (such as box or ref) and convert it into luavalue
    unsafe fn read_userdata<'lua>(
        &self,
        this: &LuaAnyUserData<'lua>,
        lua: &'lua Lua,
        userdata: &LuaAnyUserData<'lua>,
        offset: Option<isize>,
    ) -> LuaResult<LuaValue<'lua>> {
        let ptr = unsafe { get_ptr_from_userdata(userdata, offset)? };
        let value = Self::ptr_into_luavalue(self, this, lua, ptr)?;
        Ok(value)
    }

    // Write data into userdata (such as box or ref) from luavalue
    unsafe fn write_userdata<'lua>(
        &self,
        this: &LuaAnyUserData<'lua>,
        lua: &'lua Lua,
        luavalue: LuaValue<'lua>,
        userdata: LuaAnyUserData<'lua>,
        offset: Option<isize>,
    ) -> LuaResult<()> {
        let ptr = unsafe { get_ptr_from_userdata(&userdata, offset)? };
        Self::luavalue_into_ptr(self, this, lua, luavalue, ptr)?;
        Ok(())
    }
}
