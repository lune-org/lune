use std::{future::Future, net::SocketAddr, pin::Pin};

use async_tungstenite::{tungstenite::protocol::Role, WebSocketStream};
use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    service::Service as HyperService,
    Request as HyperRequest, Response as HyperResponse, StatusCode,
};

use mlua::prelude::*;
use mlua_luau_scheduler::{LuaSchedulerExt, LuaSpawnExt};

use crate::{
    server::{
        config::{ResponseConfig, ServeConfig},
        upgrade::{is_upgrade_request, make_upgrade_response},
    },
    shared::{hyper::HyperIo, request::Request, response::Response, websocket::Websocket},
};

#[derive(Debug, Clone)]
pub(super) struct Service {
    pub(super) lua: Lua,
    pub(super) address: SocketAddr, // NOTE: This must be the remote address of the connected client
    pub(super) config: ServeConfig,
}

impl HyperService<HyperRequest<Incoming>> for Service {
    type Response = HyperResponse<Full<Bytes>>;
    type Error = LuaError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn call(&self, req: HyperRequest<Incoming>) -> Self::Future {
        if is_upgrade_request(&req) {
            if let Some(handler) = self.config.handle_web_socket.clone() {
                let lua = self.lua.clone();
                return Box::pin(async move {
                    let response = match make_upgrade_response(&req) {
                        Ok(res) => res,
                        Err(err) => {
                            return Ok(HyperResponse::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Full::new(Bytes::from(err.to_string())))
                                .unwrap())
                        }
                    };

                    lua.spawn_local({
                        let lua = lua.clone();
                        async move {
                            if let Err(_err) = handle_websocket(lua, handler, req).await {
                                // TODO: Propagare the error somehow?
                            }
                        }
                    });

                    Ok(response)
                });
            }
        }

        let lua = self.lua.clone();
        let address = self.address;
        let handler = self.config.handle_request.clone();
        Box::pin(async move {
            match handle_request(lua, handler, req, address).await {
                Ok(response) => Ok(response),
                Err(_err) => {
                    // TODO: Propagare the error somehow?
                    Ok(HyperResponse::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Full::new(Bytes::from("Lune: Internal server error")))
                        .unwrap())
                }
            }
        })
    }
}

async fn handle_request(
    lua: Lua,
    handler: LuaFunction,
    request: HyperRequest<Incoming>,
    address: SocketAddr,
) -> LuaResult<HyperResponse<Full<Bytes>>> {
    let request = Request::from_incoming(request, true)
        .await?
        .with_address(address);

    let thread_id = lua.push_thread_back(handler, request)?;
    lua.track_thread(thread_id);
    lua.wait_for_thread(thread_id).await;

    let thread_res = lua
        .get_thread_result(thread_id)
        .expect("Missing handler thread result")?;

    let config = ResponseConfig::from_lua_multi(thread_res, &lua)?;
    let response = Response::try_from(config)?;

    Ok(response.into_full())
}

async fn handle_websocket(
    lua: Lua,
    handler: LuaFunction,
    request: HyperRequest<Incoming>,
) -> LuaResult<()> {
    let upgraded = hyper::upgrade::on(request).await.into_lua_err()?;

    let stream =
        WebSocketStream::from_raw_socket(HyperIo::from(upgraded), Role::Server, None).await;
    let websocket = Websocket::from(stream);

    lua.push_thread_back(handler, websocket)?;

    Ok(())
}
