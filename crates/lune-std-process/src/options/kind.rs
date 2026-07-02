use std::{fmt, process::Stdio, str::FromStr};

use mlua::prelude::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProcessSpawnOptionsStdioKind {
    // TODO: We need better more obvious names
    // for these, but that is a breaking change
    #[default]
    Default,
    Forward,
    Inherit,
    None,
}

impl ProcessSpawnOptionsStdioKind {
    pub fn all() -> &'static [Self] {
        &[Self::Default, Self::Forward, Self::Inherit, Self::None]
    }

    pub fn as_stdio(self) -> Stdio {
        match self {
            Self::None => Stdio::null(),
            Self::Forward => Stdio::inherit(),
            _ => Stdio::piped(),
        }
    }
}

impl fmt::Display for ProcessSpawnOptionsStdioKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            Self::Default => "default",
            Self::Forward => "forward",
            Self::Inherit => "inherit",
            Self::None => "none",
        };
        f.write_str(s)
    }
}

impl FromStr for ProcessSpawnOptionsStdioKind {
    type Err = LuaError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim().to_ascii_lowercase().as_str() {
            "default" => Self::Default,
            "forward" => Self::Forward,
            "inherit" => Self::Inherit,
            "none" => Self::None,
            _ => {
                return Err(LuaError::RuntimeError(format!(
                    "Invalid spawn options stdio kind - got '{}', expected one of {}",
                    s,
                    ProcessSpawnOptionsStdioKind::all()
                        .iter()
                        .map(|k| format!("'{k}'"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )));
            }
        })
    }
}

impl FromLua for ProcessSpawnOptionsStdioKind {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Nil => Ok(Self::default()),
            LuaValue::String(s) => s.to_str()?.parse(),
            _ => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "ProcessSpawnOptionsStdioKind".to_string(),
                message: Some(format!(
                    "Invalid spawn options stdio kind - expected string, got {}",
                    value.type_name()
                )),
            }),
        }
    }
}
