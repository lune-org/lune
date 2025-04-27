use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use async_channel::{unbounded, Receiver, Sender};

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
