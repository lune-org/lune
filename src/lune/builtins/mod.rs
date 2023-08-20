use std::str::FromStr;

use mlua::prelude::*;

mod fs;
mod luau;
mod process;
mod serde;
mod stdio;
mod task;

#[cfg(feature = "roblox")]
mod roblox;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum LuneBuiltin {
    Fs,
    Luau,
    Task,
    Process,
    Serde,
    Stdio,
    #[cfg(feature = "roblox")]
    Roblox,
}

impl<'lua> LuneBuiltin
where
    'lua: 'static, // FIXME: Remove static lifetime bound here when builtin libraries no longer need it
{
    pub fn name(&self) -> &'static str {
        match self {
            Self::Fs => "fs",
            Self::Luau => "luau",
            Self::Task => "task",
            Self::Process => "process",
            Self::Serde => "serde",
            Self::Stdio => "stdio",
            #[cfg(feature = "roblox")]
            Self::Roblox => "roblox",
        }
    }

    pub fn create(&self, lua: &'lua Lua) -> LuaResult<LuaMultiValue<'lua>> {
        let res = match self {
            Self::Fs => fs::create(lua),
            Self::Luau => luau::create(lua),
            Self::Task => task::create(lua),
            Self::Process => process::create(lua),
            Self::Serde => serde::create(lua),
            Self::Stdio => stdio::create(lua),
            #[cfg(feature = "roblox")]
            Self::Roblox => roblox::create(lua),
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
            "fs" => Ok(Self::Fs),
            "luau" => Ok(Self::Luau),
            "task" => Ok(Self::Task),
            "process" => Ok(Self::Process),
            "serde" => Ok(Self::Serde),
            "stdio" => Ok(Self::Stdio),
            #[cfg(feature = "roblox")]
            "roblox" => Ok(Self::Roblox),
            _ => Err(format!("Unknown builtin library '{s}'")),
        }
    }
}
