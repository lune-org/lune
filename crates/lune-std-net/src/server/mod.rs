use std::{cell::Cell, net::SocketAddr, rc::Rc};

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
pub mod upgrade;

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
            let handle_dropped = Rc::new(Cell::new(false));
            loop {
                // 1. Keep accepting new connections until we should shutdown
                let (conn, addr) = if handle_dropped.get() {
                    // 1a. Handle has been dropped, and we don't need to listen for shutdown
                    match listener.accept().await {
                        Ok(acc) => acc,
                        Err(_err) => {
                            // TODO: Propagate error somehow
                            continue;
                        }
                    }
                } else {
                    // 1b. Handle is possibly active, we must listen for shutdown
                    match either(shutdown_rx.recv(), listener.accept()).await {
                        Either::Left(Ok(())) => break,
                        Either::Left(Err(_)) => {
                            // NOTE #1: We will only get a RecvError if the serve handle is dropped,
                            // this means lua has garbage collected it and the user does not want
                            // to manually stop the server using the serve handle. Run forever.
                            handle_dropped.set(true);
                            continue;
                        }
                        Either::Right(Ok(acc)) => acc,
                        Either::Right(Err(_err)) => {
                            // TODO: Propagate error somehow
                            continue;
                        }
                    }
                };

                // 2. For each connection, spawn a new task to handle it
                lua.spawn_local({
                    let rx = shutdown_rx.clone();
                    let io = HyperIo::from(conn);

                    let mut svc = service.clone();
                    svc.address = addr;

                    let handle_dropped = Rc::clone(&handle_dropped);
                    async move {
                        let conn = Http1Builder::new()
                            .timer(HyperTimer)
                            .keep_alive(true)
                            .serve_connection(io, svc)
                            .with_upgrades();
                        if handle_dropped.get() {
                            if let Err(_err) = conn.await {
                                // TODO: Propagate error somehow
                            }
                        } else {
                            // NOTE #2: Because we use keep_alive for websockets above, we need to
                            // also manually poll this future and handle the graceful shutdown,
                            // otherwise the already accepted connection will linger and run
                            // even if the stop method has been called on the serve handle
                            pin!(conn);
                            match either(rx.recv(), conn.as_mut()).await {
                                Either::Left(Ok(())) => conn.as_mut().graceful_shutdown(),
                                Either::Left(Err(_)) => {
                                    // Same as note #1
                                    handle_dropped.set(true);
                                    if let Err(_err) = conn.await {
                                        // TODO: Propagate error somehow
                                    }
                                }
                                Either::Right(Ok(())) => {}
                                Either::Right(Err(_err)) => {
                                    // TODO: Propagate error somehow
                                }
                            }
                        }
                    }
                });
            }
        }
    });

    Ok(handle)
}
