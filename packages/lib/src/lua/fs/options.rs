use mlua::prelude::*;

pub struct FsWriteOptions {
    pub(crate) overwrite: bool,
}

impl<'lua> FromLua<'lua> for FsWriteOptions {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
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
                    to: "FsWriteOptions",
                    message: Some(format!(
                        "Invalid write options - expected boolean or table, got {}",
                        value.type_name()
                    )),
                })
            }
        })
    }
}
