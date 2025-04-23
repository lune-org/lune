use std::sync::atomic::{AtomicUsize, Ordering};

use mlua::prelude::*;

#[derive(Debug, Clone, Copy)]
pub(super) struct SvcKeys {
    key_request: &'static str,
    key_websocket: Option<&'static str>,
}

impl SvcKeys {
    pub(super) fn new(
        lua: Lua,
        handle_request: LuaFunction,
        handle_websocket: Option<LuaFunction>,
    ) -> LuaResult<Self> {
        static SERVE_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let count = SERVE_COUNTER.fetch_add(1, Ordering::Relaxed);

        // NOTE: We leak strings here, but this is an acceptable tradeoff since programs
        // generally only start one or a couple of servers and they are usually never dropped.
        // Leaking here lets us keep this struct Copy and access the request handler callbacks
        // very performantly, significantly reducing the per-request overhead of the server.
        let key_request: &'static str =
            Box::leak(format!("__net_serve_request_{count}").into_boxed_str());
        let key_websocket: Option<&'static str> = if handle_websocket.is_some() {
            Some(Box::leak(
                format!("__net_serve_websocket_{count}").into_boxed_str(),
            ))
        } else {
            None
        };

        lua.set_named_registry_value(key_request, handle_request)?;
        if let Some(key) = key_websocket {
            lua.set_named_registry_value(key, handle_websocket.unwrap())?;
        }

        Ok(Self {
            key_request,
            key_websocket,
        })
    }

    pub(super) fn has_websocket_handler(&self) -> bool {
        self.key_websocket.is_some()
    }

    pub(super) fn request_handler(&self, lua: &Lua) -> LuaResult<LuaFunction> {
        lua.named_registry_value(self.key_request)
    }

    pub(super) fn websocket_handler(&self, lua: &Lua) -> LuaResult<Option<LuaFunction>> {
        self.key_websocket
            .map(|key| lua.named_registry_value(key))
            .transpose()
    }
}
