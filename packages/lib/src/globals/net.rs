use std::collections::HashMap;

use mlua::prelude::*;

use console::style;
use hyper::Server;
use tokio::{sync::mpsc, task};

use crate::{
    lua::{
        // net::{NetWebSocketClient, NetWebSocketServer},
        net::{NetClient, NetClientBuilder, NetLocalExec, NetService, RequestConfig, ServeConfig},
        task::TaskScheduler,
    },
    utils::{net::get_request_user_agent_header, table::TableBuilder},
};

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    // Create a reusable client for performing our
    // web requests and store it in the lua registry,
    // allowing us to reuse headers and internal structs
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

async fn net_request<'a>(lua: &'static Lua, config: RequestConfig<'a>) -> LuaResult<LuaTable<'a>> {
    // Create and send the request
    let client: NetClient = lua.named_registry_value("net.client")?;
    let mut request = client.request(config.method, &config.url);
    for (header, value) in config.headers {
        request = request.header(header.to_str()?, value.to_str()?);
    }
    let res = request
        .body(config.body.unwrap_or_default())
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

async fn net_socket<'a>(_lua: &'static Lua, _url: String) -> LuaResult<LuaTable> {
    Err(LuaError::RuntimeError(
        "Client websockets are not yet implemented".to_string(),
    ))
    // let (ws, _) = tokio_tungstenite::connect_async(url)
    //     .await
    //     .map_err(LuaError::external)?;
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
    let server_request_callback = lua.create_registry_value(config.handle_request)?;
    let server_websocket_callback = config.handle_web_socket.map(|handler| {
        lua.create_registry_value(handler)
            .expect("Failed to store websocket handler")
    });
    let sched = lua.app_data_ref::<&TaskScheduler>().unwrap();
    // Bind first to make sure that we can bind to this address
    let bound = match Server::try_bind(&([127, 0, 0, 1], port).into()) {
        Err(e) => {
            return Err(LuaError::external(format!(
                "Failed to bind to localhost on port {port}\n{}",
                format!("{e}").replace(
                    "error creating server listener: ",
                    &format!("{}", style("> ").dim())
                )
            )));
        }
        Ok(bound) => bound,
    };
    // Register a background task to prevent the task scheduler from
    // exiting early and start up our web server on the bound address
    let task = sched.register_background_task();
    let server = bound
        .http1_only(true) // Web sockets can only use http1
        .http1_keepalive(true) // Web sockets must be kept alive
        .executor(NetLocalExec)
        .serve(NetService::new(
            lua,
            server_request_callback,
            server_websocket_callback,
        ))
        .with_graceful_shutdown(async move {
            task.unregister(Ok(()));
            shutdown_rx
                .recv()
                .await
                .expect("Server was stopped instantly");
            shutdown_rx.close();
        });
    // Spawn a new tokio task so we don't block
    task::spawn_local(server);
    // Create a new read-only table that contains methods
    // for manipulating server behavior and shutting it down
    let handle_stop = move |_, _: ()| match shutdown_tx.try_send(()) {
        Ok(_) => Ok(()),
        Err(_) => Err(LuaError::RuntimeError(
            "Server has already been stopped".to_string(),
        )),
    };
    TableBuilder::new(lua)?
        .with_function("stop", handle_stop)?
        .build_readonly()
}
