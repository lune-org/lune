use mlua::prelude::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct TcpConfig {
    pub tls: Option<bool>,
    pub ttl: Option<u32>,
}

impl FromLua for TcpConfig {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        if let LuaValue::Nil = value {
            Ok(TcpConfig::default())
        } else if let LuaValue::Boolean(tls) = value {
            Ok(TcpConfig {
                tls: Some(tls),
                ttl: None,
            })
        } else if let LuaValue::Table(tab) = value {
            let mut this = TcpConfig::default();

            if let Some(tls) = tab.get::<Option<_>>("tls")? {
                this.tls = Some(tls);
            }
            if let Some(ttl) = tab.get::<Option<_>>("ttl")? {
                this.ttl = Some(ttl);
            }

            Ok(this)
        } else {
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: String::from("TcpConfig"),
                message: None,
            })
        }
    }
}
