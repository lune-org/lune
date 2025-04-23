use mlua::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct FsWriteOptions {
    pub(crate) overwrite: bool,
}

impl FromLua for FsWriteOptions {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        Ok(match value {
            LuaValue::Nil => Self { overwrite: false },
            LuaValue::Boolean(b) => Self { overwrite: b },
            LuaValue::Table(t) => {
                let overwrite: Option<bool> = t.get("overwrite")?;
                Self {
                    overwrite: overwrite.unwrap_or(false),
                }
            }
            _ => {
                return Err(LuaError::FromLuaConversionError {
                    from: value.type_name(),
                    to: "FsWriteOptions".to_string(),
                    message: Some(format!(
                        "Invalid write options - expected boolean or table, got {}",
                        value.type_name()
                    )),
                })
            }
        })
    }
}
