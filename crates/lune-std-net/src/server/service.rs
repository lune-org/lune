use std::{future::Future, net::SocketAddr, pin::Pin};

use http_body_util::Full;
use hyper::{
    body::{Bytes, Incoming},
    service::Service as HyperService,
    Request as HyperRequest, Response as HyperResponse,
};

use mlua::prelude::*;
use mlua_luau_scheduler::LuaSchedulerExt;

use crate::{
    server::config::{ResponseConfig, ServeConfig},
    shared::{request::Request, response::Response},
};

#[derive(Debug, Clone)]
pub(super) struct Service {
    pub(super) lua: Lua,
    pub(super) address: SocketAddr,
    pub(super) config: ServeConfig,
}

impl HyperService<HyperRequest<Incoming>> for Service {
    type Response = HyperResponse<Full<Bytes>>;
    type Error = LuaError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn call(&self, req: HyperRequest<Incoming>) -> Self::Future {
        let lua = self.lua.clone();
        let config = self.config.clone();

        Box::pin(async move {
            let handler = config.handle_request.clone();
            let request = Request::from_incoming(req, true).await?;

            let thread_id = lua.push_thread_back(handler, request)?;
            lua.track_thread(thread_id);
            lua.wait_for_thread(thread_id).await;

            let thread_res = lua
                .get_thread_result(thread_id)
                .expect("Missing handler thread result")?;

            let config = ResponseConfig::from_lua_multi(thread_res, &lua)?;
            let response = Response::try_from(config)?;

            Ok(response.as_full())
        })
    }
}
