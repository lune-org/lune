use mlua::prelude::*;

use super::kind::ProcessSpawnOptionsStdioKind;

#[derive(Debug, Clone, Default)]
pub struct ProcessSpawnOptionsStdio {
    pub stdout: ProcessSpawnOptionsStdioKind,
    pub stderr: ProcessSpawnOptionsStdioKind,
    pub stdin: Option<Vec<u8>>,
}

impl From<ProcessSpawnOptionsStdioKind> for ProcessSpawnOptionsStdio {
    fn from(value: ProcessSpawnOptionsStdioKind) -> Self {
        Self {
            stdout: value,
            stderr: value,
            ..Default::default()
        }
    }
}

impl FromLua for ProcessSpawnOptionsStdio {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Nil => Ok(Self::default()),
            LuaValue::String(s) => {
                Ok(ProcessSpawnOptionsStdioKind::from_lua(LuaValue::String(s), lua)?.into())
            }
            LuaValue::Table(t) => {
                let mut this = Self::default();

                if let Some(stdin) = t.get("stdin")? {
                    this.stdin = stdin;
                }

                if let Some(stdout) = t.get("stdout")? {
                    this.stdout = stdout;
                }

                if let Some(stderr) = t.get("stderr")? {
                    this.stderr = stderr;
                }

                Ok(this)
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "ProcessSpawnOptionsStdio".to_string(),
                message: Some(format!(
                    "Invalid spawn options stdio - expected string or table, got {}",
                    value.type_name()
                )),
            }),
        }
    }
}
