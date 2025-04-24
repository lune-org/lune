use std::process::ExitStatus;

use async_channel::{unbounded, Receiver, Sender};
use async_process::Child as AsyncChild;
use futures_util::{select, FutureExt};

use mlua::prelude::*;
use mlua_luau_scheduler::LuaSpawnExt;

use lune_utils::TableBuilder;

use super::{ChildReader, ChildWriter};

#[derive(Debug, Clone)]
pub struct Child {
    stdin: ChildWriter,
    stdout: ChildReader,
    stderr: ChildReader,
    kill_tx: Sender<()>,
    status_rx: Receiver<Option<ExitStatus>>,
}

impl Child {
    pub fn new(lua: &Lua, mut child: AsyncChild) -> Self {
        let stdin = ChildWriter::from(child.stdin.take());
        let stdout = ChildReader::from(child.stdout.take());
        let stderr = ChildReader::from(child.stderr.take());

        // NOTE: Kill channel is zero size, status is very small
        // and implements Copy, unbounded will be just fine here
        let (kill_tx, kill_rx) = unbounded();
        let (status_tx, status_rx) = unbounded();
        lua.spawn(handle_child(child, kill_rx, status_tx)).detach();

        Self {
            stdin,
            stdout,
            stderr,
            kill_tx,
            status_rx,
        }
    }
}

impl LuaUserData for Child {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("stdin", |_, this| Ok(this.stdin.clone()));
        fields.add_field_method_get("stdout", |_, this| Ok(this.stdout.clone()));
        fields.add_field_method_get("stderr", |_, this| Ok(this.stderr.clone()));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("kill", |_, this, (): ()| {
            let _ = this.kill_tx.try_send(());
            Ok(())
        });
        methods.add_async_method("status", |lua, this, (): ()| {
            let rx = this.status_rx.clone();
            async move {
                let status = rx.recv().await.ok().flatten();
                let code = status.and_then(|c| c.code()).unwrap_or(9);
                TableBuilder::new(lua.clone())?
                    .with_value("ok", code == 0)?
                    .with_value("code", code)?
                    .build_readonly()
            }
        });
    }
}

async fn handle_child(
    mut child: AsyncChild,
    kill_rx: Receiver<()>,
    status_tx: Sender<Option<ExitStatus>>,
) {
    let status = select! {
        s = child.status().fuse() => s.ok(), // FUTURE: Propagate this error somehow?
        _ = kill_rx.recv().fuse() => {
            let _ = child.kill(); // Will only error if already killed
            None
        }
    };

    // Will only error if there are no receivers waiting for the status
    let _ = status_tx.send(status).await;
}
