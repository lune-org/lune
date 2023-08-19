use mlua::prelude::*;

#[derive(Clone)]
pub(super) enum LuaThreadOrFunction<'lua> {
    Thread(LuaThread<'lua>),
    Function(LuaFunction<'lua>),
}

impl<'lua> LuaThreadOrFunction<'lua> {
    pub(super) fn into_thread(self, lua: &'lua Lua) -> LuaResult<LuaThread<'lua>> {
        match self {
            Self::Thread(t) => Ok(t),
            Self::Function(f) => lua.create_thread(f),
        }
    }
}

impl<'lua> FromLua<'lua> for LuaThreadOrFunction<'lua> {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Thread(t) => Ok(Self::Thread(t)),
            LuaValue::Function(f) => Ok(Self::Function(f)),
            value => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "LuaThreadOrFunction",
                message: Some("Expected thread or function".to_string()),
            }),
        }
    }
}
