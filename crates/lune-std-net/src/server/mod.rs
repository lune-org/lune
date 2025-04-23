use std::net::SocketAddr;

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, pin};

use mlua::prelude::*;
use mlua_luau_scheduler::LuaSpawnExt;

use lune_utils::TableBuilder;

use super::config::ServeConfig;

mod keys;
mod request;
mod response;
mod service;

use keys::SvcKeys;
use service::Svc;

pub async fn serve(lua: Lua, port: u16, config: ServeConfig) -> LuaResult<LuaTable> {
    let addr: SocketAddr = (config.address, port).into();
    let listener = TcpListener::bind(addr).await?;

    let lua_svc = lua.clone();
    let lua_inner = lua.clone();

    let keys = SvcKeys::new(lua.clone(), config.handle_request, config.handle_web_socket)?;
    let svc = Svc {
        lua: lua_svc,
        addr,
        keys,
    };

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    lua.spawn_local(async move {
        let mut shutdown_rx_outer = shutdown_rx.clone();
        loop {
            // Create futures for accepting new connections and shutting down
            let fut_shutdown = shutdown_rx_outer.changed();
            let fut_accept = async {
                let stream = match listener.accept().await {
                    Err(_) => return,
                    Ok((s, _)) => s,
                };

                let io = TokioIo::new(stream);
                let svc = svc.clone();
                let mut shutdown_rx_inner = shutdown_rx.clone();

                lua_inner.spawn_local(async move {
                    let conn = http1::Builder::new()
                        .keep_alive(true) // Web sockets need this
                        .serve_connection(io, svc)
                        .with_upgrades();
                    // NOTE: Because we need to use keep_alive for websockets, we need to
                    // also manually poll this future and handle the shutdown signal here
                    pin!(conn);
                    tokio::select! {
                        _ = conn.as_mut() => {}
                        _ = shutdown_rx_inner.changed() => {
                            conn.as_mut().graceful_shutdown();
                        }
                    }
                });
            };

            // Wait for either a new connection or a shutdown signal
            tokio::select! {
                () = fut_accept => {}
                res = fut_shutdown => {
                    // NOTE: We will only get a RecvError here if the serve handle is dropped,
                    // this means lua has garbage collected it and the user does not want
                    // to manually stop the server using the serve handle. Run forever.
                    if res.is_ok() {
                        break;
                    }
                }
            }
        }
    });

    TableBuilder::new(lua)?
        .with_value("ip", addr.ip().to_string())?
        .with_value("port", addr.port())?
        .with_function("stop", move |_, (): ()| match shutdown_tx.send(true) {
            Ok(()) => Ok(()),
            Err(_) => Err(LuaError::runtime("Server already stopped")),
        })?
        .build_readonly()
}
