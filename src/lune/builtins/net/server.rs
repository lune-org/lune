use std::{
    collections::HashMap,
    future::Future,
    net::SocketAddr,
    pin::Pin,
    rc::{Rc, Weak},
    str::FromStr,
};

use http::request::Parts;
use http_body_util::{BodyExt, Full};
use hyper::{
    body::{Bytes, Incoming},
    header::{HeaderName, HeaderValue},
    server::conn::http1,
    service::Service,
    HeaderMap, Request, Response,
};
use hyper_tungstenite::{is_upgrade_request, upgrade};
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, pin};

use mlua::prelude::*;
use mlua_luau_scheduler::{LuaSchedulerExt, LuaSpawnExt};

use crate::lune::util::TableBuilder;

use super::{config::ServeConfig, websocket::NetWebSocket};

struct LuaRequest {
    _remote_addr: SocketAddr,
    head: Parts,
    body: Vec<u8>,
}

impl LuaUserData for LuaRequest {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("method", |_, this| {
            Ok(this.head.method.as_str().to_string())
        });

        fields.add_field_method_get("path", |_, this| Ok(this.head.uri.path().to_string()));

        fields.add_field_method_get("query", |_, this| {
            let query: HashMap<String, String> = this
                .head
                .uri
                .query()
                .unwrap_or_default()
                .split('&')
                .filter_map(|q| q.split_once('='))
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            Ok(query)
        });

        fields.add_field_method_get("headers", |_, this| {
            let headers: HashMap<String, Vec<u8>> = this
                .head
                .headers
                .iter()
                .map(|(k, v)| (k.as_str().to_string(), v.as_bytes().to_vec()))
                .collect();
            Ok(headers)
        });

        fields.add_field_method_get("body", |lua, this| lua.create_string(&this.body));
    }
}

#[derive(Debug, Clone, Copy)]
enum LuaResponseKind {
    PlainText,
    Table,
}

struct LuaResponse {
    kind: LuaResponseKind,
    status: u16,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
}

impl LuaResponse {
    fn into_response(self) -> LuaResult<Response<Full<Bytes>>> {
        Ok(match self.kind {
            LuaResponseKind::PlainText => Response::builder()
                .status(200)
                .header("Content-Type", "text/plain")
                .body(Full::new(Bytes::from(self.body.unwrap())))
                .into_lua_err()?,
            LuaResponseKind::Table => {
                let mut response = Response::builder()
                    .status(self.status)
                    .body(Full::new(Bytes::from(self.body.unwrap_or_default())))
                    .into_lua_err()?;
                response.headers_mut().extend(self.headers);
                response
            }
        })
    }
}

impl FromLua<'_> for LuaResponse {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        match value {
            // Plain strings from the handler are plaintext responses
            LuaValue::String(s) => Ok(Self {
                kind: LuaResponseKind::PlainText,
                status: 200,
                headers: HeaderMap::new(),
                body: Some(s.as_bytes().to_vec()),
            }),
            // Tables are more detailed responses with potential status, headers, body
            LuaValue::Table(t) => {
                let status: Option<u16> = t.get("status")?;
                let headers: Option<LuaTable> = t.get("headers")?;
                let body: Option<LuaString> = t.get("body")?;

                let mut headers_map = HeaderMap::new();
                if let Some(headers) = headers {
                    for pair in headers.pairs::<String, LuaString>() {
                        let (h, v) = pair?;
                        let name = HeaderName::from_str(&h).into_lua_err()?;
                        let value = HeaderValue::from_bytes(v.as_bytes()).into_lua_err()?;
                        headers_map.insert(name, value);
                    }
                }

                let body_bytes = body.map(|s| s.as_bytes().to_vec());

                Ok(Self {
                    kind: LuaResponseKind::Table,
                    status: status.unwrap_or(200),
                    headers: headers_map,
                    body: body_bytes,
                })
            }
            // Anything else is an error
            value => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "NetServeResponse",
                message: None,
            }),
        }
    }
}

struct Svc {
    lua: Rc<Lua>,
    addr: SocketAddr,
    handler_request: LuaRegistryKey,
    handler_websocket: LuaRegistryKey,
    has_websocket_handler: bool,
}

impl Svc {
    fn clone_registry_keys(&self) -> (LuaRegistryKey, LuaRegistryKey) {
        let cloned_request = self
            .lua
            .registry_value::<LuaFunction>(&self.handler_request)
            .expect("Failed to clone registry value");
        let cloned_websocket = self
            .lua
            .registry_value::<Option<LuaFunction>>(&self.handler_websocket)
            .expect("Failed to clone registry value");

        let stored_request = self
            .lua
            .create_registry_value(cloned_request)
            .expect("Failed to clone registry value");
        let stored_websocket = self
            .lua
            .create_registry_value(cloned_websocket)
            .expect("Failed to clone registry value");

        (stored_request, stored_websocket)
    }
}

impl Clone for Svc {
    fn clone(&self) -> Self {
        let (handler_request, handler_websocket) = self.clone_registry_keys();
        Self {
            lua: self.lua.clone(),
            addr: self.addr,
            handler_request,
            handler_websocket,
            has_websocket_handler: self.has_websocket_handler,
        }
    }
}

impl Service<Request<Incoming>> for Svc {
    type Response = Response<Full<Bytes>>;
    type Error = LuaError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let addr = self.addr;
        let lua = self.lua.clone();

        let (handler_request, handler_websocket) = self.clone_registry_keys();

        if self.has_websocket_handler && is_upgrade_request(&req) {
            Box::pin(async move {
                let (res, sock) = upgrade(req, None).into_lua_err()?;

                let lua_inner = lua.clone();
                lua.spawn_local(async move {
                    let sock = sock.await.unwrap();
                    let lua_sock = NetWebSocket::new(sock).into_lua_table(&lua_inner).unwrap();

                    let handler_websocket = lua_inner
                        .registry_value::<LuaFunction>(&handler_websocket)
                        .unwrap();

                    lua_inner
                        .push_thread_back(handler_websocket, lua_sock)
                        .unwrap();
                });

                Ok(res)
            })
        } else {
            let (head, body) = req.into_parts();

            Box::pin(async move {
                let handler_request = lua.registry_value::<LuaFunction>(&handler_request)?;

                let body = body.collect().await.into_lua_err()?.to_bytes().to_vec();

                let lua_req = LuaRequest {
                    _remote_addr: addr,
                    head,
                    body,
                };

                let thread_id = lua.push_thread_back(handler_request, lua_req)?;
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

pub async fn serve<'lua>(
    lua: &'lua Lua,
    port: u16,
    config: ServeConfig<'lua>,
) -> LuaResult<LuaTable<'lua>> {
    let addr: SocketAddr = (config.address, port).into();
    let listener = TcpListener::bind(addr).await?;

    let (lua_inner, lua_inner_2) = {
        let rc = lua
            .app_data_ref::<Weak<Lua>>()
            .expect("Missing weak lua ref")
            .upgrade()
            .expect("Lua was dropped unexpectedly");
        (Rc::clone(&rc), rc)
    };

    let svc = Svc {
        lua: lua_inner,
        addr,
        has_websocket_handler: config.handle_web_socket.is_some(),
        handler_request: lua.create_registry_value(config.handle_request)?,
        handler_websocket: lua.create_registry_value(config.handle_web_socket)?,
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

                lua_inner_2.spawn_local(async move {
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
                _ = fut_accept => {}
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
        .with_function("stop", move |lua, _: ()| match shutdown_tx.send(true) {
            Ok(_) => Ok(()),
            Err(_) => Err(LuaError::runtime("Server already stopped")),
        })?
        .build_readonly()
}
