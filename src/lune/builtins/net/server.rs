use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use hyper::{
    server::{conn::AddrIncoming, Builder},
    service::{make_service_fn, service_fn},
    Response, Server,
};

use mlua::prelude::*;
use tokio::sync::mpsc;

use crate::{
    lune::{scheduler::Scheduler, util::TableBuilder},
    LuneError,
};

use super::{config::ServeConfig, processing::ProcessedRequest, response::NetServeResponse};

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

    // Communicate between background thread(s) and main lua thread using mpsc
    let (tx_request, mut rx_request) = mpsc::channel::<ProcessedRequest>(64);
    let (tx_websocket, mut rx_websocket) = mpsc::channel::<()>(64);
    let tx_request_arc = Arc::new(tx_request);
    let tx_websocket_arc = Arc::new(tx_websocket);

    // Create our background service which will accept
    // requests, do some processing, then forward to lua
    let hyper_make_service = make_service_fn(move |_| {
        let tx_request = Arc::clone(&tx_request_arc);
        let tx_websocket = Arc::clone(&tx_websocket_arc);

        let handler = service_fn(move |req| {
            // TODO: Check if we should upgrade to a
            // websocket, handle the request differently
            let tx_request = Arc::clone(&tx_request);
            let tx_websocket = Arc::clone(&tx_websocket);
            async move {
                let processed = ProcessedRequest::from_request(req).await?;
                if (tx_request.send(processed).await).is_err() {
                    return Err(LuaError::runtime("Lua handler is busy"));
                }
                // TODO: Wait for response from lua
                let res = Response::new("TODO".to_string());
                Ok::<_, LuaError>(res)
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
                shutdown_rx.recv().await;
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

            // NOTE: The closure here is not really necessary, we
            // make the closure so that we can use the `?` operator
            let handle_req_or_sock = || async {
                match (req, sock) {
                    (None, None) => Ok::<_, LuaError>(true),
                    (Some(req), _) => {
                        let req_table = req.into_lua_table(lua)?;
                        let req_handler = config.handle_request.clone();

                        let thread_id = sched.push_back(lua, req_handler, req_table)?;
                        let thread_res = sched.wait_for_thread(lua, thread_id).await?;

                        // TODO: Send response back to other thread somehow
                        let handler_res = NetServeResponse::from_lua_multi(thread_res, lua)?;

                        Ok(false)
                    }
                    (_, Some(_sock)) => {
                        let sock_handler = config
                            .handle_web_socket
                            .as_ref()
                            .cloned()
                            .expect("Got web socket but web socket handler is missing");

                        // TODO: Convert websocket into lua websocket struct, give as args
                        let thread_id = sched.push_back(lua, sock_handler, ())?;

                        // NOTE: Web socket handler does not need to send any
                        // response back, the websocket upgrade response is
                        // automatically sent above in the background thread(s)
                        sched.wait_for_thread(lua, thread_id).await?;

                        Ok(false)
                    }
                }
            };

            match handle_req_or_sock().await {
                Ok(true) => break,
                Ok(false) => continue,
                Err(e) => eprintln!("{}", LuneError::from(e)),
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
