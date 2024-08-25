use core::ffi::c_int;

use libffi::middle::Type;
use mlua::prelude::*;

use super::super::c_type::{CType, PtrHandle};

impl PtrHandle for CType<c_int> {
    fn luavalue_into_ptr(value: LuaValue, ptr: *mut ()) -> LuaResult<()> {
        let value = match value {
            LuaValue::Integer(t) => t,
            _ => {
                return Err(LuaError::external(format!(
                    "Integer expected, got {}",
                    value.type_name()
                )))
            }
        } as c_int;
        unsafe {
            *(ptr.cast::<c_int>()) = value;
        }
        Ok(())
    }
    fn ptr_into_luavalue(lua: &Lua, ptr: *mut ()) -> LuaResult<LuaValue> {
        let value = unsafe { (*ptr.cast::<c_int>()).into_lua(lua)? };
        Ok(value)
    }
}

impl CType<c_int> {
    fn new() -> LuaResult<Self> {
        Self::new_with_libffi_type(Type::c_int(), Some("int"))
    }
}

pub fn get_export(lua: &Lua) -> LuaResult<(&'static str, LuaAnyUserData)> {
    Ok(("int", lua.create_userdata(CType::<c_int>::new()?)?))
}
