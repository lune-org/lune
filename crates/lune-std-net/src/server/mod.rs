use std::net::SocketAddr;

use async_net::TcpListener;
use futures_lite::pin;
use hyper::server::conn::http1::Builder as Http1Builder;

use mlua::prelude::*;
use mlua_luau_scheduler::LuaSpawnExt;

use crate::{
    server::{config::ServeConfig, handle::ServeHandle, service::Service},
    shared::{
        futures::{either, Either},
        hyper::{HyperIo, HyperTimer},
    },
};

pub mod config;
pub mod handle;
pub mod service;

/**
    Starts an HTTP server using the given port and configuration.

    Returns a `ServeHandle` that can be used to gracefully stop the server.
*/
pub async fn serve(lua: Lua, port: u16, config: ServeConfig) -> LuaResult<ServeHandle> {
    let address = SocketAddr::from((config.address, port));
    let service = Service {
        lua: lua.clone(),
        address,
        config,
    };

    let listener = TcpListener::bind(address).await?;
    let (handle, shutdown_rx) = ServeHandle::new(address);

    lua.spawn_local({
        let lua = lua.clone();
        async move {
            let mut running_forever = false;
            loop {
                let accepted = if running_forever {
                    listener.accept().await
                } else {
                    match either(shutdown_rx.recv(), listener.accept()).await {
                        Either::Left(res) => {
                            if res.is_ok() {
                                break;
                            }
                            // NOTE: We will only get a RecvError if the serve handle is dropped,
                            // this means lua has garbage collected it and the user does not want
                            // to manually stop the server using the serve handle. Run forever.
                            running_forever = true;
                            continue;
                        }
                        Either::Right(acc) => acc,
                    }
                };

                let (conn, addr) = match accepted {
                    Ok((conn, addr)) => (conn, addr),
                    Err(err) => {
                        eprintln!("Error while accepting connection: {err}");
                        continue;
                    }
                };

                lua.spawn_local({
                    let rx = shutdown_rx.clone();
                    let io = HyperIo::from(conn);
                    let mut svc = service.clone();
                    svc.address = addr;
                    async move {
                        let conn = Http1Builder::new()
                            .timer(HyperTimer)
                            .keep_alive(true)
                            .serve_connection(io, svc)
                            .with_upgrades();
                        // NOTE: Because we use keep_alive for websockets above, we need to
                        // also manually poll this future and handle the shutdown signal here
                        pin!(conn);
                        match either(rx.recv(), conn.as_mut()).await {
                            Either::Left(_) => conn.as_mut().graceful_shutdown(),
                            Either::Right(_) => {}
                        }
                    }
                });
            }
        }
    });

    Ok(handle)
}
