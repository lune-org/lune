use std::str::FromStr;

use mlua::prelude::*;

mod luau;
mod stdio;
mod task;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum LuneBuiltin {
    Luau,
    Task,
    Stdio,
}

impl<'lua> LuneBuiltin
where
    'lua: 'static, // FIXME: Remove static lifetime bound here when builtin libraries no longer need it
{
    pub fn name(&self) -> &'static str {
        match self {
            Self::Luau => "luau",
            Self::Task => "task",
            Self::Stdio => "stdio",
        }
    }

    pub fn create(&self, lua: &'lua Lua) -> LuaResult<LuaMultiValue<'lua>> {
        let res = match self {
            Self::Luau => luau::create(lua),
            Self::Task => task::create(lua),
            Self::Stdio => stdio::create(lua),
        };
        match res {
            Ok(v) => v.into_lua_multi(lua),
            Err(e) => Err(e.context(format!(
                "Failed to create builtin library '{}'",
                self.name()
            ))),
        }
    }
}

impl FromStr for LuneBuiltin {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "luau" => Ok(Self::Luau),
            "task" => Ok(Self::Task),
            "stdio" => Ok(Self::Stdio),
            _ => Err(format!("Unknown builtin library '{s}'")),
        }
    }
}
