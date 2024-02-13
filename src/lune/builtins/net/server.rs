use std::{convert::Infallible, net::SocketAddr};

use http_body_util::Full;
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, spawn, sync::mpsc::channel};

use hyper::{
    body::{Bytes, Incoming},
    server::conn::http1,
    service::service_fn,
    Request, Response,
};

use mlua::prelude::*;

use crate::lune::util::TableBuilder;

use super::config::ServeConfig;

const SERVER_IMPL_LUA: &str = r#"
spawn(function()
    while true do
        local id, request, socket, exit = server:next()
        if exit then
            break
        end
        spawn(function()
            if socket ~= nil then
                local handler = server:getRequestHandler()
                local response = handler(request)
                server:respond(id, response)
            elseif request ~= nil then
                local handler = server:getWebsocketHandler()
                handler(socket)
            end
        end)
    end
end)
"#;

pub(super) async fn serve<'lua>(
    lua: &'lua Lua,
    port: u16,
    config: ServeConfig<'lua>,
) -> LuaResult<LuaTable<'lua>> {
    let addr = SocketAddr::from((config.address, port));
    let listener = TcpListener::bind(addr).await.map_err(|e| {
        LuaError::external(format!(
            "Failed to bind to {addr}\n{}",
            e.to_string()
                .replace("error creating server listener: ", "> ")
        ))
    })?;

    // Spawn a new task to accept incoming connections + listening for shutdown
    let (shutdown_tx, mut shutdown_rx) = channel::<()>(1);
    spawn(async move {
        loop {
            tokio::select! {
                // If we receive a shutdown signal, break the loop
                _ = shutdown_rx.recv() => break,
                // Each connection gets its own task that forwards to lua
                accepted = listener.accept() => {
                    match accepted {
                        Err(e) => println!("Error accepting connection: {e}"),
                        Ok((s, _)) => {
                            let io = TokioIo::new(s);
                            spawn(async move {
                                if let Err(err) = http1::Builder::new()
                                    .serve_connection(io, service_fn(|_| async move {
                                        // TODO: Forward to lua somehow
                                        Ok::<_, Infallible>(Response::new(Full::new(Bytes::from("Hello, World!"))))
                                    }))
                                    .await
                                {
                                    println!("Error serving connection: {err:?}");
                                }
                            });
                        }
                    }
                }
            }
        }
    });

    // Create a new read-only table that contains methods
    // for manipulating server behavior and shutting it down
    let handle_stop = move |_, _: ()| match shutdown_tx.try_send(()) {
        Err(_) => Err(LuaError::runtime("Server has already been stopped")),
        Ok(_) => Ok(()),
    };

    TableBuilder::new(lua)?
        .with_function("stop", handle_stop)?
        .build_readonly()
}
