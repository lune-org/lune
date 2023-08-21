use std::{convert::Infallible, net::SocketAddr};

use hyper::{
    server::{conn::AddrIncoming, Builder},
    service::{make_service_fn, service_fn},
    Response, Server,
};
use mlua::prelude::*;
use tokio::sync::mpsc;

use crate::lune::{scheduler::Scheduler, util::TableBuilder};

use super::config::ServeConfig;

pub(super) fn bind_to_localhost(port: u16) -> LuaResult<Builder<AddrIncoming>> {
    let addr = match SocketAddr::try_from(([127, 0, 0, 1], port)) {
        Ok(a) => a,
        Err(e) => {
            return Err(LuaError::external(format!(
                "Failed to bind to localhost on port {port}\n{e}"
            )))
        }
    };
    match Server::try_bind(&addr) {
        Ok(b) => Ok(b),
        Err(e) => Err(LuaError::external(format!(
            "Failed to bind to localhost on port {port}\n{}",
            e.to_string()
                .replace("error creating server listener: ", "> ")
        ))),
    }
}

pub(super) fn create_server<'lua>(
    lua: &'lua Lua,
    sched: &'lua Scheduler,
    config: ServeConfig<'lua>,
    builder: Builder<AddrIncoming>,
) -> LuaResult<LuaTable<'lua>>
where
    'lua: 'static, // FIXME: Get rid of static lifetime bound here
{
    // Note that we need to use a mpsc here and not
    // a oneshot channel since we move the sender
    // into our table with the stop function
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

    // Spawn a scheduler background task, communicate using mpsc
    // channels, do any heavy lifting possible in background thread
    let (tx_request, mut rx_request) = mpsc::channel::<()>(64);
    let (tx_websocket, mut rx_websocket) = mpsc::channel::<()>(64);
    sched.spawn(async move {
        let result = builder
            .serve(make_service_fn(|_| async move {
                Ok::<_, Infallible>(service_fn(|_req| async move {
                    // TODO: Send this request back to lua
                    let res = Response::new("TODO".to_string());
                    Ok::<_, Infallible>(res)
                }))
            }))
            .with_graceful_shutdown(async move {
                shutdown_rx.recv().await;
            });
        if let Err(e) = result.await {
            eprintln!("Net serve error: {e}")
        }
    });

    // Spawn a local thread with access to lua, this will get
    // requests and sockets to handle using our lua handlers
    sched.spawn_local(async move {
        loop {
            let (req, sock) = tokio::select! {
                req = rx_request.recv() => (req, None),
                sock = rx_websocket.recv() => (None, sock),
            };
            if req.is_none() && sock.is_none() {
                break;
            }
            if let Some(_req) = req {
                // TODO: Convert request into lua request struct
                let thread_id = sched
                    .push_back(lua, config.handle_request.clone(), ())
                    .expect("Failed to spawn net serve handler");
                // TODO: Send response back to other thread somehow
                match sched.wait_for_thread(lua, thread_id).await {
                    Err(e) => eprintln!("Net serve handler error: {e}"),
                    Ok(v) => println!("Net serve handler result: {v:?}"),
                };
            }
            if let Some(_sock) = sock {
                let handle_web_socket = config
                    .handle_web_socket
                    .as_ref()
                    .expect("Got web socket but web socket handler is missing");
                // TODO: Convert request into lua request struct
                let thread_id = sched
                    .push_back(lua, handle_web_socket.clone(), ())
                    .expect("Failed to spawn net websocket handler");
                // TODO: Send response back to other thread somehow
                match sched.wait_for_thread(lua, thread_id).await {
                    Err(e) => eprintln!("Net websocket handler error: {e}"),
                    Ok(v) => println!("Net websocket handler result: {v:?}"),
                };
            }
        }
    });

    // Create a new read-only table that contains methods
    // for manipulating server behavior and shutting it down
    let handle_stop = move |_, _: ()| match shutdown_tx.try_send(()) {
        Ok(_) => Ok(()),
        Err(_) => Err(LuaError::RuntimeError(
            "Server has already been stopped".to_string(),
        )),
    };
    TableBuilder::new(lua)?
        .with_function("stop", handle_stop)?
        .build_readonly()
}
