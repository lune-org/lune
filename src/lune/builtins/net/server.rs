use std::{
    collections::HashMap,
    convert::Infallible,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use hyper::{
    server::{conn::AddrIncoming, Builder},
    service::{make_service_fn, service_fn},
    Server,
};

use hyper_tungstenite::{is_upgrade_request, upgrade, HyperWebsocket};
use mlua::prelude::*;
use tokio::sync::{mpsc, oneshot, Mutex};

use crate::lune::{
    scheduler::Scheduler,
    util::{futures::yield_forever, traits::LuaEmitErrorExt, TableBuilder},
};

use super::{
    config::ServeConfig, processing::ProcessedRequest, response::NetServeResponse,
    websocket::NetWebSocket,
};

pub(super) fn bind_to_addr(address: Ipv4Addr, port: u16) -> LuaResult<Builder<AddrIncoming>> {
    let addr = SocketAddr::from((address, port));

    match Server::try_bind(&addr) {
        Ok(b) => Ok(b),
        Err(e) => Err(LuaError::external(format!(
            "Failed to bind to {addr}\n{}",
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

    // Communicate between background thread(s) and main lua thread using mpsc and oneshot
    let (tx_request, mut rx_request) = mpsc::channel::<ProcessedRequest>(64);
    let (tx_websocket, mut rx_websocket) = mpsc::channel::<HyperWebsocket>(64);
    let tx_request_arc = Arc::new(tx_request);
    let tx_websocket_arc = Arc::new(tx_websocket);

    let response_senders = Arc::new(Mutex::new(HashMap::new()));
    let response_senders_bg = Arc::clone(&response_senders);
    let response_senders_lua = Arc::clone(&response_senders_bg);

    // Create our background service which will accept
    // requests, do some processing, then forward to lua
    let has_websocket_handler = config.handle_web_socket.is_some();
    let hyper_make_service = make_service_fn(move |_| {
        let tx_request = Arc::clone(&tx_request_arc);
        let tx_websocket = Arc::clone(&tx_websocket_arc);
        let response_senders = Arc::clone(&response_senders_bg);

        let handler = service_fn(move |mut req| {
            let tx_request = Arc::clone(&tx_request);
            let tx_websocket = Arc::clone(&tx_websocket);
            let response_senders = Arc::clone(&response_senders);
            async move {
                // FUTURE: Improve error messages when lua is busy and queue is full
                if has_websocket_handler && is_upgrade_request(&req) {
                    let (response, ws) = match upgrade(&mut req, None) {
                        Err(_) => return Err(LuaError::runtime("Failed to upgrade websocket")),
                        Ok(v) => v,
                    };
                    if (tx_websocket.send(ws).await).is_err() {
                        return Err(LuaError::runtime("Lua handler is busy"));
                    }
                    Ok(response)
                } else {
                    let processed = ProcessedRequest::from_request(req).await?;
                    let request_id = processed.id;
                    if (tx_request.send(processed).await).is_err() {
                        return Err(LuaError::runtime("Lua handler is busy"));
                    }
                    let (response_tx, response_rx) = oneshot::channel::<NetServeResponse>();
                    response_senders
                        .lock()
                        .await
                        .insert(request_id, response_tx);
                    match response_rx.await {
                        Err(_) => Err(LuaError::runtime("Internal Server Error")),
                        Ok(r) => r.into_response(),
                    }
                }
            }
        });

        async move { Ok::<_, Infallible>(handler) }
    });

    // Start up our service
    sched.spawn(async move {
        let result = builder
            .http1_only(true) // Web sockets can only use http1
            .http1_keepalive(true) // Web sockets must be kept alive
            .serve(hyper_make_service)
            .with_graceful_shutdown(async move {
                if shutdown_rx.recv().await.is_none() {
                    // The channel was closed, meaning the serve handle
                    // was garbage collected by lua without being used
                    yield_forever().await;
                }
            });
        if let Err(e) = result.await {
            eprintln!("Net serve error: {e}")
        }
    });

    // Spawn a local thread with access to lua and the same lifetime
    sched.spawn_local(async move {
        loop {
            // Wait for either a request or a websocket to handle,
            // if we got neither it means both channels were dropped
            // and our server has stopped, either gracefully or panic
            let (req, sock) = tokio::select! {
                req = rx_request.recv() => (req, None),
                sock = rx_websocket.recv() => (None, sock),
            };
            if req.is_none() && sock.is_none() {
                break;
            }

            // NOTE: The closure here is not really necessary, we
            // make the closure so that we can use the `?` operator
            // and make a catch-all for errors in spawn_local below
            let handle_request = config.handle_request.clone();
            let handle_web_socket = config.handle_web_socket.clone();
            let response_senders = Arc::clone(&response_senders_lua);
            let response_fut = async move {
                match (req, sock) {
                    (Some(req), _) => {
                        let req_id = req.id;
                        let req_table = req.into_lua_table(lua)?;

                        let thread_id = sched.push_back(lua, handle_request, req_table)?;
                        let thread_res = sched.wait_for_thread(lua, thread_id).await?;

                        let response = NetServeResponse::from_lua_multi(thread_res, lua)?;
                        let response_sender = response_senders
                            .lock()
                            .await
                            .remove(&req_id)
                            .expect("Response channel was removed unexpectedly");

                        // NOTE: We ignore the error here, if the sender is no longer
                        // being listened to its because our client disconnected during
                        // handler being called, which is fine and should not emit errors
                        response_sender.send(response).ok();

                        Ok(())
                    }
                    (_, Some(sock)) => {
                        let sock = sock.await.into_lua_err()?;

                        let sock_handler = handle_web_socket
                            .as_ref()
                            .cloned()
                            .expect("Got web socket but web socket handler is missing");
                        let sock_table = NetWebSocket::new(sock).into_lua_table(lua)?;

                        // NOTE: Web socket handler does not need to send any
                        // response back, the websocket upgrade response is
                        // automatically sent above in the background thread(s)
                        let thread_id = sched.push_back(lua, sock_handler, sock_table)?;
                        let _thread_res = sched.wait_for_thread(lua, thread_id).await?;

                        Ok(())
                    }
                    _ => unreachable!(),
                }
            };

            /*
                NOTE: It is currently not possible to spawn new background tasks from within
                another background task with the Lune scheduler since they are locked behind a
                mutex and we also need that mutex locked to be able to run a background task...

                We need to do some work to make it so our unordered futures queues do
                not require locking and then we can replace the following bit of code:

                sched.spawn_local(async {
                    if let Err(e) = response_fut.await {
                        lua.emit_error(e);
                    }
                });
            */
            if let Err(e) = response_fut.await {
                lua.emit_error(e);
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
