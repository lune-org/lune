use mlua::prelude::*;

use super::TaskReference;

/*
    Proxy enum to deal with both threads & functions
*/

#[derive(Debug, Clone)]
pub enum LuaThreadOrFunction<'lua> {
    Thread(LuaThread<'lua>),
    Function(LuaFunction<'lua>),
}

impl<'lua> LuaThreadOrFunction<'lua> {
    pub fn into_thread(self, lua: &'lua Lua) -> LuaResult<LuaThread<'lua>> {
        match self {
            Self::Thread(t) => Ok(t),
            Self::Function(f) => lua.create_thread(f),
        }
    }
}

impl<'lua> From<LuaThread<'lua>> for LuaThreadOrFunction<'lua> {
    fn from(value: LuaThread<'lua>) -> Self {
        Self::Thread(value)
    }
}

impl<'lua> From<LuaFunction<'lua>> for LuaThreadOrFunction<'lua> {
    fn from(value: LuaFunction<'lua>) -> Self {
        Self::Function(value)
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
                message: Some(format!(
                    "Expected thread or function, got '{}'",
                    value.type_name()
                )),
            }),
        }
    }
}

impl<'lua> IntoLua<'lua> for LuaThreadOrFunction<'lua> {
    fn into_lua(self, _: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        match self {
            Self::Thread(t) => Ok(LuaValue::Thread(t)),
            Self::Function(f) => Ok(LuaValue::Function(f)),
        }
    }
}

/*
    Proxy enum to deal with both threads & task scheduler task references
*/

#[derive(Debug, Clone)]
pub enum LuaThreadOrTaskReference<'lua> {
    Thread(LuaThread<'lua>),
    TaskReference(TaskReference),
}

impl<'lua> From<LuaThread<'lua>> for LuaThreadOrTaskReference<'lua> {
    fn from(value: LuaThread<'lua>) -> Self {
        Self::Thread(value)
    }
}

impl<'lua> From<TaskReference> for LuaThreadOrTaskReference<'lua> {
    fn from(value: TaskReference) -> Self {
        Self::TaskReference(value)
    }
}

impl<'lua> FromLua<'lua> for LuaThreadOrTaskReference<'lua> {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        let tname = value.type_name();
        match value {
            LuaValue::Thread(t) => Ok(Self::Thread(t)),
            LuaValue::UserData(u) => {
                if let Ok(task) =
                    LuaUserDataRef::<TaskReference>::from_lua(LuaValue::UserData(u), lua)
                {
                    Ok(Self::TaskReference(*task))
                } else {
                    Err(LuaError::FromLuaConversionError {
                        from: tname,
                        to: "thread",
                        message: Some(format!("Expected thread, got '{tname}'")),
                    })
                }
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: tname,
                to: "thread",
                message: Some(format!("Expected thread, got '{tname}'")),
            }),
        }
    }
}

impl<'lua> IntoLua<'lua> for LuaThreadOrTaskReference<'lua> {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        match self {
            Self::TaskReference(t) => t.into_lua(lua),
            Self::Thread(t) => Ok(LuaValue::Thread(t)),
        }
    }
}
