use mlua::prelude::*;

pub struct ServeConfig<'a> {
    pub handle_request: LuaFunction<'a>,
    pub handle_web_socket: Option<LuaFunction<'a>>,
}

impl<'lua> FromLua<'lua> for ServeConfig<'lua> {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        let message = match &value {
            LuaValue::Function(f) => {
                return Ok(ServeConfig {
                    handle_request: f.clone(),
                    handle_web_socket: None,
                })
            }
            LuaValue::Table(t) => {
                let handle_request: Option<LuaFunction> = t.raw_get("handleRequest")?;
                let handle_web_socket: Option<LuaFunction> = t.raw_get("handleWebSocket")?;
                if handle_request.is_some() || handle_web_socket.is_some() {
                    return Ok(ServeConfig {
                        handle_request: handle_request.unwrap_or_else(|| {
                            let chunk = r#"
                            return {
                                status = 426,
                                body = "Upgrade Required",
                                headers = {
                                    Upgrade = "websocket",
                                },
                            }
                            "#;
                            lua.load(chunk)
                                .into_function()
                                .expect("Failed to create default http responder function")
                        }),
                        handle_web_socket,
                    });
                } else {
                    Some("Missing handleRequest and / or handleWebSocket".to_string())
                }
            }
            _ => None,
        };
        Err(LuaError::FromLuaConversionError {
            from: value.type_name(),
            to: "ServeConfig",
            message,
        })
    }
}
