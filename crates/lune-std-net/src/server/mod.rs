use std::net::SocketAddr;

use async_net::TcpListener;
use hyper::server::conn::http1::Builder as Http1Builder;

use mlua::prelude::*;
use mlua_luau_scheduler::LuaSpawnExt;

use crate::{
    server::{config::ServeConfig, service::Service},
    shared::hyper::{HyperIo, HyperTimer},
};

pub mod config;
pub mod service;

/**
    Starts an HTTP server using the given port and configuration.
*/
pub async fn serve(lua: Lua, port: u16, config: ServeConfig) -> LuaResult<()> {
    let address = SocketAddr::from((config.address, port));
    let service = Service {
        lua: lua.clone(),
        address,
        config,
    };

    let listener = TcpListener::bind(address).await?;

    lua.spawn_local({
        let lua = lua.clone();
        async move {
            loop {
                let (connection, _addr) = match listener.accept().await {
                    Ok((connection, addr)) => (connection, addr),
                    Err(err) => {
                        eprintln!("Error while accepting connection: {err}");
                        continue;
                    }
                };

                lua.spawn_local({
                    let service = service.clone();
                    async move {
                        let result = Http1Builder::new()
                            .timer(HyperTimer)
                            .keep_alive(true) // Needed for websockets
                            .serve_connection(HyperIo::from(connection), service)
                            .with_upgrades()
                            .await;
                        if let Err(err) = result {
                            eprintln!("Error while responding to request: {err}");
                        }
                    }
                });
            }
        }
    });

    Ok(())
}
