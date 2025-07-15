use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use async_channel::{Receiver, Sender, unbounded};

use lune_utils::TableBuilder;
use mlua::prelude::*;

#[derive(Debug, Clone)]
pub struct ServeHandle {
    addr: SocketAddr,
    shutdown: Arc<AtomicBool>,
    sender: Sender<()>,
}

impl ServeHandle {
    pub fn new(addr: SocketAddr) -> (Self, Receiver<()>) {
        let (sender, receiver) = unbounded();
        let this = Self {
            addr,
            shutdown: Arc::new(AtomicBool::new(false)),
            sender,
        };
        (this, receiver)
    }

    // TODO: Remove this in the next major release to use colon/self
    // based call syntax and userdata implementation below instead
    pub fn into_lua_table(self, lua: Lua) -> LuaResult<LuaTable> {
        let shutdown = self.shutdown.clone();
        let sender = self.sender.clone();
        TableBuilder::new(lua)?
            .with_value("ip", self.addr.ip().to_string())?
            .with_value("port", self.addr.port())?
            .with_function("stop", move |_, ()| {
                if shutdown.load(Ordering::SeqCst) {
                    Err(LuaError::runtime("Server already stopped"))
                } else {
                    shutdown.store(true, Ordering::SeqCst);
                    sender.try_send(()).ok();
                    sender.close();
                    Ok(())
                }
            })?
            .build()
    }
}

impl LuaUserData for ServeHandle {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("ip", |_, this| Ok(this.addr.ip().to_string()));
        fields.add_field_method_get("port", |_, this| Ok(this.addr.port()));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("stop", |_, this, ()| {
            if this.shutdown.load(Ordering::SeqCst) {
                Err(LuaError::runtime("Server already stopped"))
            } else {
                this.shutdown.store(true, Ordering::SeqCst);
                this.sender.try_send(()).ok();
                this.sender.close();
                Ok(())
            }
        });
    }
}
