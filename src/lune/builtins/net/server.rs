use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use mlua::prelude::*;

use hyper::{body::to_bytes, server::conn::AddrStream, service::Service};
use hyper::{Body, Request, Response};
use hyper_tungstenite::{is_upgrade_request as is_ws_upgrade_request, upgrade as ws_upgrade};
use tokio::task;

use crate::lune::{
    scheduler::Scheduler,
    util::{traits::LuaEmitErrorExt, TableBuilder},
};

use super::{NetServeResponse, NetWebSocket};

// Hyper service implementation for net, lots of boilerplate here
// but make_svc and make_svc_function do not work for what we need

pub struct NetServiceInner(
    &'static Lua,
    Arc<LuaRegistryKey>,
    Arc<Option<LuaRegistryKey>>,
);

impl Service<Request<Body>> for NetServiceInner {
    type Response = Response<Body>;
    type Error = LuaError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let lua = self.0;
        if self.2.is_some() && is_ws_upgrade_request(&req) {
            // Websocket upgrade request + websocket handler exists,
            // we should now upgrade this connection to a websocket
            // and then call our handler with a new socket object
            let kopt = self.2.clone();
            let key = kopt.as_ref().as_ref().unwrap();
            let handler: LuaFunction = lua.registry_value(key).expect("Missing websocket handler");
            let (response, ws) = ws_upgrade(&mut req, None).expect("Failed to upgrade websocket");
            // This should be spawned as a scheduler task, otherwise
            // the scheduler may exit early and cancel this even though what
            // we want here is a long-running task that keeps the program alive
            let sched = lua
                .app_data_ref::<&Scheduler>()
                .expect("Lua struct is missing scheduler");
            sched.spawn_local(async move {
                // Create our new full websocket object, then
                // schedule our handler to get called asap
                let res = async move {
                    let ws = ws.await.into_lua_err()?;
                    let sock = NetWebSocket::new(ws).into_lua_table(lua)?;
                    sched.push_front(
                        lua,
                        lua.create_thread(handler)?,
                        LuaMultiValue::from_vec(vec![LuaValue::Table(sock)]),
                    )
                };
                if let Err(e) = res.await {
                    lua.emit_error(e);
                }
            });
            Box::pin(async move { Ok(response) })
        } else {
            // Got a normal http request or no websocket handler
            // exists, just call the http request handler
            let key = self.1.clone();
            let (parts, body) = req.into_parts();
            Box::pin(async move {
                // Convert request body into bytes, extract handler
                let bytes = to_bytes(body).await.into_lua_err()?;
                let handler: LuaFunction = lua.registry_value(&key)?;
                // Create a readonly table for the request query params
                let query_params = TableBuilder::new(lua)?
                    .with_values(
                        parts
                            .uri
                            .query()
                            .unwrap_or_default()
                            .split('&')
                            .filter_map(|q| q.split_once('='))
                            .collect(),
                    )?
                    .build_readonly()?;
                // Do the same for headers
                let header_map = TableBuilder::new(lua)?
                    .with_values(
                        parts
                            .headers
                            .iter()
                            .map(|(name, value)| {
                                (name.to_string(), value.to_str().unwrap().to_string())
                            })
                            .collect(),
                    )?
                    .build_readonly()?;
                // Create a readonly table with request info to pass to the handler
                let request = TableBuilder::new(lua)?
                    .with_value("path", parts.uri.path())?
                    .with_value("query", query_params)?
                    .with_value("method", parts.method.as_str())?
                    .with_value("headers", header_map)?
                    .with_value("body", lua.create_string(&bytes)?)?
                    .build_readonly()?;
                let response: LuaResult<NetServeResponse> = handler.call(request);
                // Return successful response, or emit any error using pretty formatting
                lua.emit_error(match response {
                    Ok(r) => match r.into_response() {
                        Ok(res) => return Ok(res),
                        Err(err) => err,
                    },
                    Err(err) => err,
                });
                Ok(Response::builder()
                    .status(500)
                    .body(Body::from("Internal Server Error"))
                    .unwrap())
            })
        }
    }
}

pub struct NetService(
    &'static Lua,
    Arc<LuaRegistryKey>,
    Arc<Option<LuaRegistryKey>>,
);

impl NetService {
    pub fn new(
        lua: &'static Lua,
        callback_http: LuaRegistryKey,
        callback_websocket: Option<LuaRegistryKey>,
    ) -> Self {
        Self(lua, Arc::new(callback_http), Arc::new(callback_websocket))
    }
}

impl Service<&AddrStream> for NetService {
    type Response = NetServiceInner;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: &AddrStream) -> Self::Future {
        let lua = self.0;
        let key1 = self.1.clone();
        let key2 = self.2.clone();
        Box::pin(async move { Ok(NetServiceInner(lua, key1, key2)) })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NetLocalExec;

impl<F> hyper::rt::Executor<F> for NetLocalExec
where
    F: std::future::Future + 'static, // not requiring `Send`
{
    fn execute(&self, fut: F) {
        task::spawn_local(fut);
    }
}
