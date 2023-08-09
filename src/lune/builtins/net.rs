use std::collections::HashMap;

use mlua::prelude::*;

use console::style;
use hyper::{
    header::{CONTENT_ENCODING, CONTENT_LENGTH},
    Server,
};
use tokio::{sync::mpsc, task};

use crate::lune::lua::{
    net::{
        NetClient, NetClientBuilder, NetLocalExec, NetService, NetWebSocket, RequestConfig,
        ServeConfig,
    },
    serde::{decompress, CompressDecompressFormat, EncodeDecodeConfig, EncodeDecodeFormat},
    table::TableBuilder,
    task::{TaskScheduler, TaskSchedulerAsyncExt},
};

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    // Create a reusable client for performing our
    // web requests and store it in the lua registry,
    // allowing us to reuse headers and internal structs
    let client = NetClientBuilder::new()
        .headers(&[("User-Agent", create_user_agent_header())])?
        .build()?;
    lua.set_named_registry_value("net.client", client)?;
    // Create the global table for net
    TableBuilder::new(lua)?
        .with_function("jsonEncode", net_json_encode)?
        .with_function("jsonDecode", net_json_decode)?
        .with_async_function("request", net_request)?
        .with_async_function("socket", net_socket)?
        .with_async_function("serve", net_serve)?
        .with_function("urlEncode", net_url_encode)?
        .with_function("urlDecode", net_url_decode)?
        .build_readonly()
}

fn create_user_agent_header() -> String {
    let (github_owner, github_repo) = env!("CARGO_PKG_REPOSITORY")
        .trim_start_matches("https://github.com/")
        .split_once('/')
        .unwrap();
    format!("{github_owner}-{github_repo}-cli")
}

fn net_json_encode<'a>(
    lua: &'static Lua,
    (val, pretty): (LuaValue<'a>, Option<bool>),
) -> LuaResult<LuaString<'a>> {
    EncodeDecodeConfig::from((EncodeDecodeFormat::Json, pretty.unwrap_or_default()))
        .serialize_to_string(lua, val)
}

fn net_json_decode<'a>(lua: &'static Lua, json: LuaString<'a>) -> LuaResult<LuaValue<'a>> {
    EncodeDecodeConfig::from(EncodeDecodeFormat::Json).deserialize_from_string(lua, json)
}

async fn net_request<'a>(lua: &'static Lua, config: RequestConfig<'a>) -> LuaResult<LuaTable<'a>> {
    // Create and send the request
    let client: LuaUserDataRef<NetClient> = lua.named_registry_value("net.client")?;
    let mut request = client.request(config.method, &config.url);
    for (query, value) in config.query {
        request = request.query(&[(query.to_str()?, value.to_str()?)]);
    }
    for (header, value) in config.headers {
        request = request.header(header.to_str()?, value.to_str()?);
    }
    let res = request
        .body(config.body.unwrap_or_default())
        .send()
        .await
        .into_lua_err()?;
    // Extract status, headers
    let res_status = res.status().as_u16();
    let res_status_text = res.status().canonical_reason();
    let mut res_headers = res
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_string(),
                value.to_str().unwrap().to_owned(),
            )
        })
        .collect::<HashMap<String, String>>();
    // Read response bytes
    let mut res_bytes = res.bytes().await.into_lua_err()?.to_vec();
    // Check for extra options, decompression
    if config.options.decompress {
        // NOTE: Header names are guaranteed to be lowercase because of the above
        // transformations of them into the hashmap, so we can compare directly
        let format = res_headers.iter().find_map(|(name, val)| {
            if name == CONTENT_ENCODING.as_str() {
                CompressDecompressFormat::detect_from_header_str(val)
            } else {
                None
            }
        });
        if let Some(format) = format {
            res_bytes = decompress(format, res_bytes).await?;
            let content_encoding_header_str = CONTENT_ENCODING.as_str();
            let content_length_header_str = CONTENT_LENGTH.as_str();
            res_headers.retain(|name, _| {
                name != content_encoding_header_str && name != content_length_header_str
            });
        }
    }
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
    let (ws, _) = tokio_tungstenite::connect_async(url).await.into_lua_err()?;
    NetWebSocket::new(ws).into_lua_table(lua)
}

async fn net_serve<'a>(
    lua: &'static Lua,
    (port, config): (u16, ServeConfig<'a>),
) -> LuaResult<LuaTable<'a>> {
    // Note that we need to use a mpsc here and not
    // a oneshot channel since we move the sender
    // into our table with the stop function
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    let server_request_callback = lua.create_registry_value(config.handle_request)?;
    let server_websocket_callback = config.handle_web_socket.map(|handler| {
        lua.create_registry_value(handler)
            .expect("Failed to store websocket handler")
    });
    let sched = lua
        .app_data_ref::<&TaskScheduler>()
        .expect("Missing task scheduler - make sure it is added as a lua app data before the first scheduler resumption");
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

fn net_url_encode<'a>(
    lua: &'static Lua,
    (lua_string, as_binary): (LuaString<'a>, Option<bool>),
) -> LuaResult<LuaValue<'a>> {
    if matches!(as_binary, Some(true)) {
        urlencoding::encode_binary(lua_string.as_bytes()).into_lua(lua)
    } else {
        urlencoding::encode(lua_string.to_str()?).into_lua(lua)
    }
}

fn net_url_decode<'a>(
    lua: &'static Lua,
    (lua_string, as_binary): (LuaString<'a>, Option<bool>),
) -> LuaResult<LuaValue<'a>> {
    if matches!(as_binary, Some(true)) {
        urlencoding::decode_binary(lua_string.as_bytes()).into_lua(lua)
    } else {
        urlencoding::decode(lua_string.to_str()?)
            .map_err(|e| LuaError::RuntimeError(format!("Encountered invalid encoding - {e}")))?
            .into_lua(lua)
    }
}
