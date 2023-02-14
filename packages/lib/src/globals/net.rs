use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use mlua::prelude::*;

use hyper::{body::to_bytes, server::conn::AddrStream, service::Service};
use hyper::{Body, Request, Response, Server};
use hyper_tungstenite::{is_upgrade_request as is_ws_upgrade_request, upgrade as ws_upgrade};

use reqwest::Method;
use tokio::{sync::mpsc, task};

use crate::{
    lua::{
        // net::{NetWebSocketClient, NetWebSocketServer},
        net::{NetClient, NetClientBuilder, ServeConfig},
        task::TaskScheduler,
    },
    utils::{net::get_request_user_agent_header, table::TableBuilder},
};

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    // Create a reusable client for performing our
    // web requests and store it in the lua registry
    let client = NetClientBuilder::new()
        .headers(&[("User-Agent", get_request_user_agent_header())])?
        .build()?;
    lua.set_named_registry_value("net.client", client)?;
    // Create the global table for net
    TableBuilder::new(lua)?
        .with_function("jsonEncode", net_json_encode)?
        .with_function("jsonDecode", net_json_decode)?
        .with_async_function("request", net_request)?
        .with_async_function("socket", net_socket)?
        .with_async_function("serve", net_serve)?
        .build_readonly()
}

fn net_json_encode(_: &'static Lua, (val, pretty): (LuaValue, Option<bool>)) -> LuaResult<String> {
    if let Some(true) = pretty {
        serde_json::to_string_pretty(&val).map_err(LuaError::external)
    } else {
        serde_json::to_string(&val).map_err(LuaError::external)
    }
}

fn net_json_decode(lua: &'static Lua, json: String) -> LuaResult<LuaValue> {
    let json: serde_json::Value = serde_json::from_str(&json).map_err(LuaError::external)?;
    lua.to_value(&json)
}

async fn net_request<'a>(lua: &'static Lua, config: LuaValue<'a>) -> LuaResult<LuaTable<'a>> {
    let client: NetClient = lua.named_registry_value("net.client")?;
    // Extract stuff from config and make sure its all valid
    let (url, method, headers, body) = match config {
        LuaValue::String(s) => {
            let url = s.to_string_lossy().to_string();
            let method = "GET".to_string();
            Ok((url, method, HashMap::new(), None))
        }
        LuaValue::Table(tab) => {
            // Extract url
            let url = match tab.raw_get::<_, LuaString>("url") {
                Ok(config_url) => Ok(config_url.to_string_lossy().to_string()),
                Err(_) => Err(LuaError::RuntimeError(
                    "Missing 'url' in request config".to_string(),
                )),
            }?;
            // Extract method
            let method = match tab.raw_get::<_, LuaString>("method") {
                Ok(config_method) => config_method.to_string_lossy().trim().to_ascii_uppercase(),
                Err(_) => "GET".to_string(),
            };
            // Extract headers
            let headers = match tab.raw_get::<_, LuaTable>("headers") {
                Ok(config_headers) => {
                    let mut lua_headers = HashMap::new();
                    for pair in config_headers.pairs::<LuaString, LuaString>() {
                        let (key, value) = pair?.to_owned();
                        lua_headers.insert(key, value);
                    }
                    lua_headers
                }
                Err(_) => HashMap::new(),
            };
            // Extract body
            let body = match tab.raw_get::<_, LuaString>("body") {
                Ok(config_body) => Some(config_body.as_bytes().to_owned()),
                Err(_) => None,
            };
            Ok((url, method, headers, body))
        }
        value => Err(LuaError::RuntimeError(format!(
            "Invalid request config - expected string or table, got {}",
            value.type_name()
        ))),
    }?;
    // Convert method string into proper enum
    let method = method.trim().to_ascii_uppercase();
    let method = match method.as_ref() {
        "GET" => Ok(Method::GET),
        "POST" => Ok(Method::POST),
        "PUT" => Ok(Method::PUT),
        "DELETE" => Ok(Method::DELETE),
        "HEAD" => Ok(Method::HEAD),
        "OPTIONS" => Ok(Method::OPTIONS),
        "PATCH" => Ok(Method::PATCH),
        _ => Err(LuaError::RuntimeError(format!(
            "Invalid request config method '{}'",
            &method
        ))),
    }?;
    // Create and send the request
    let mut request = client.request(method, &url);
    for (header, value) in headers {
        request = request.header(header.to_str()?, value.to_str()?);
    }
    let res = request
        .body(body.unwrap_or_default())
        .send()
        .await
        .map_err(LuaError::external)?;
    // Extract status, headers
    let res_status = res.status().as_u16();
    let res_status_text = res.status().canonical_reason();
    let res_headers = res
        .headers()
        .iter()
        .map(|(name, value)| (name.to_string(), value.to_str().unwrap().to_owned()))
        .collect::<HashMap<String, String>>();
    // Read response bytes
    let res_bytes = res.bytes().await.map_err(LuaError::external)?;
    // Construct and return a readonly lua table with results
    TableBuilder::new(lua)?
        .with_value("ok", (200..300).contains(&res_status))?
        .with_value("statusCode", res_status)?
        .with_value("statusMessage", res_status_text)?
        .with_value("headers", res_headers)?
        .with_value("body", lua.create_string(&res_bytes)?)?
        .build_readonly()
}

async fn net_socket<'a>(lua: &'static Lua, url: String) -> LuaResult<LuaTable> {
    let (ws, _) = tokio_tungstenite::connect_async(url)
        .await
        .map_err(LuaError::external)?;
    Err(LuaError::RuntimeError(
        "Client websockets are not yet implemented".to_string(),
    ))
    // let sock = NetWebSocketClient::from(ws);
    // let table = sock.into_lua_table(lua)?;
    // Ok(table)
}

async fn net_serve<'a>(
    lua: &'static Lua,
    (port, config): (u16, ServeConfig<'a>),
) -> LuaResult<LuaTable<'a>> {
    if config.handle_web_socket.is_some() {
        return Err(LuaError::RuntimeError(
            "Server websockets are not yet implemented".to_string(),
        ));
    }
    // Note that we need to use a mpsc here and not
    // a oneshot channel since we move the sender
    // into our table with the stop function
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    let server_request_callback = Arc::new(lua.create_registry_value(config.handle_request)?);
    let server_websocket_callback = Arc::new(config.handle_web_socket.map(|handler| {
        lua.create_registry_value(handler)
            .expect("Failed to store websocket handler")
    }));
    // Register a background task to prevent
    // the task scheduler from exiting early
    let sched = lua.app_data_mut::<&TaskScheduler>().unwrap();
    let task = sched.register_background_task();
    let server = Server::bind(&([127, 0, 0, 1], port).into())
        .http1_only(true)
        .http1_keepalive(true)
        .executor(LocalExec)
        .serve(MakeNetService(
            lua,
            server_request_callback,
            server_websocket_callback,
        ))
        .with_graceful_shutdown(async move {
            shutdown_rx
                .recv()
                .await
                .expect("Server was stopped instantly");
            shutdown_rx.close();
            task.unregister(Ok(()));
        });
    // Spawn a new tokio task so we don't block
    task::spawn_local(server);
    // Create a new read-only table that contains methods
    // for manipulating server behavior and shutting it down
    let handle_stop = move |_, _: ()| {
        if shutdown_tx.try_send(()).is_err() {
            Err(LuaError::RuntimeError(
                "Server has already been stopped".to_string(),
            ))
        } else {
            Ok(())
        }
    };
    TableBuilder::new(lua)?
        .with_function("stop", handle_stop)?
        .build_readonly()
}

// Hyper service implementation for net, lots of boilerplate here
// but make_svc and make_svc_function do not work for what we need

pub struct NetService(
    &'static Lua,
    Arc<LuaRegistryKey>,
    Arc<Option<LuaRegistryKey>>,
);

impl Service<Request<Body>> for NetService {
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
            // TODO: This should be spawned as part of the scheduler,
            // the scheduler may exit early and cancel this even though what
            // we want here is a long-running task that keeps the program alive
            task::spawn_local(async move {
                // Create our new full websocket object, then
                // schedule our handler to get called asap
                let ws = ws.await.map_err(LuaError::external)?;
                // let sock = NetWebSocketServer::from(ws);
                // let table = sock.into_lua_table(lua)?;
                // let sched = lua.app_data_mut::<&TaskScheduler>().unwrap();
                // sched.schedule_current_resume(
                //     LuaValue::Function(handler),
                //     LuaMultiValue::from_vec(vec![LuaValue::Table(table)]),
                // )
                Ok::<_, LuaError>(())
            });
            Box::pin(async move { Ok(response) })
        } else {
            // Got a normal http request or no websocket handler
            // exists, just call the http request handler
            let key = self.1.clone();
            let (parts, body) = req.into_parts();
            Box::pin(async move {
                // Convert request body into bytes, extract handler
                // function & lune message sender to use later
                let bytes = to_bytes(body).await.map_err(LuaError::external)?;
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
                match handler.call(request) {
                    // Plain strings from the handler are plaintext responses
                    Ok(LuaValue::String(s)) => Ok(Response::builder()
                        .status(200)
                        .header("Content-Type", "text/plain")
                        .body(Body::from(s.as_bytes().to_vec()))
                        .unwrap()),
                    // Tables are more detailed responses with potential status, headers, body
                    Ok(LuaValue::Table(t)) => {
                        let status = t.get::<_, Option<u16>>("status")?.unwrap_or(200);
                        let mut resp = Response::builder().status(status);

                        if let Some(headers) = t.get::<_, Option<LuaTable>>("headers")? {
                            for pair in headers.pairs::<String, LuaString>() {
                                let (h, v) = pair?;
                                resp = resp.header(&h, v.as_bytes());
                            }
                        }

                        let body = t
                            .get::<_, Option<LuaString>>("body")?
                            .map(|b| Body::from(b.as_bytes().to_vec()))
                            .unwrap_or_else(Body::empty);

                        Ok(resp.body(body).unwrap())
                    }
                    // If the handler returns an error, generate a 5xx response
                    Err(_) => {
                        // TODO: Send above error to task scheduler so that it can emit properly
                        Ok(Response::builder()
                            .status(500)
                            .body(Body::from("Internal Server Error"))
                            .unwrap())
                    }
                    // If the handler returns a value that is of an invalid type,
                    // this should also be an error, so generate a 5xx response
                    Ok(value) => {
                        // TODO: Send below error to task scheduler so that it can emit properly
                        let _ = LuaError::RuntimeError(format!(
                            "Expected net serve handler to return a value of type 'string' or 'table', got '{}'",
                            value.type_name()
                        ));
                        Ok(Response::builder()
                            .status(500)
                            .body(Body::from("Internal Server Error"))
                            .unwrap())
                    }
                }
            })
        }
    }
}

struct MakeNetService(
    &'static Lua,
    Arc<LuaRegistryKey>,
    Arc<Option<LuaRegistryKey>>,
);

impl Service<&AddrStream> for MakeNetService {
    type Response = NetService;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: &AddrStream) -> Self::Future {
        let lua = self.0;
        let key1 = self.1.clone();
        let key2 = self.2.clone();
        Box::pin(async move { Ok(NetService(lua, key1, key2)) })
    }
}

#[derive(Clone, Copy, Debug)]
struct LocalExec;

impl<F> hyper::rt::Executor<F> for LocalExec
where
    F: std::future::Future + 'static, // not requiring `Send`
{
    fn execute(&self, fut: F) {
        task::spawn_local(fut);
    }
}
