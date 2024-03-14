use std::{future::Future, net::SocketAddr, pin::Pin, rc::Rc};

use http_body_util::{BodyExt, Full};
use hyper::{
    body::{Bytes, Incoming},
    service::Service,
    Request, Response,
};
use hyper_tungstenite::{is_upgrade_request, upgrade};

use mlua::prelude::*;
use mlua_luau_scheduler::{LuaSchedulerExt, LuaSpawnExt};

use super::{
    super::websocket::NetWebSocket, keys::SvcKeys, request::LuaRequest, response::LuaResponse,
};

#[derive(Debug, Clone)]
pub(super) struct Svc {
    pub(super) lua: Rc<Lua>,
    pub(super) addr: SocketAddr,
    pub(super) keys: SvcKeys,
}

impl Service<Request<Incoming>> for Svc {
    type Response = Response<Full<Bytes>>;
    type Error = LuaError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let lua = self.lua.clone();
        let addr = self.addr;
        let keys = self.keys;

        if keys.has_websocket_handler() && is_upgrade_request(&req) {
            Box::pin(async move {
                let (res, sock) = upgrade(req, None).into_lua_err()?;

                let lua_inner = lua.clone();
                lua.spawn_local(async move {
                    let sock = sock.await.unwrap();
                    let lua_sock = NetWebSocket::new(sock);
                    let lua_tab = lua_sock.into_lua_table(&lua_inner).unwrap();

                    let handler_websocket: LuaFunction =
                        keys.websocket_handler(&lua_inner).unwrap().unwrap();

                    lua_inner
                        .push_thread_back(handler_websocket, lua_tab)
                        .unwrap();
                });

                Ok(res)
            })
        } else {
            let (head, body) = req.into_parts();

            Box::pin(async move {
                let handler_request: LuaFunction = keys.request_handler(&lua).unwrap();

                let body = body.collect().await.into_lua_err()?;
                let body = body.to_bytes().to_vec();

                let lua_req = LuaRequest {
                    _remote_addr: addr,
                    head,
                    body,
                };
                let lua_req_table = lua_req.into_lua_table(&lua)?;

                let thread_id = lua.push_thread_back(handler_request, lua_req_table)?;
                lua.track_thread(thread_id);
                lua.wait_for_thread(thread_id).await;
                let thread_res = lua
                    .get_thread_result(thread_id)
                    .expect("Missing handler thread result")?;

                LuaResponse::from_lua_multi(thread_res, &lua)?.into_response()
            })
        }
    }
}
