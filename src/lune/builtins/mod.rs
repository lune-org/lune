use std::str::FromStr;

use mlua::prelude::*;

mod task;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum LuneBuiltin {
    Task,
}

impl<'lua> LuneBuiltin
where
    'lua: 'static, // FIXME: Remove static lifetime bound here when builtin libraries no longer need it
{
    pub fn name(&self) -> &'static str {
        match self {
            Self::Task => "task",
        }
    }

    pub fn create(&self, lua: &'lua Lua) -> LuaResult<LuaMultiValue<'lua>> {
        let res = match self {
            Self::Task => task::create(lua),
        };
        match res {
            Ok(v) => Ok(v.into_lua_multi(lua)?),
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
            "task" => Ok(Self::Task),
            _ => Err(format!("Unknown builtin library '{s}'")),
        }
    }
}
