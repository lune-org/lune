use std::str::FromStr;

use mlua::prelude::*;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_ENCODING};

use lune_std_serde::{decompress, CompressDecompressFormat};
use lune_utils::TableBuilder;

use super::{config::RequestConfig, util::header_map_to_table};

const REGISTRY_KEY: &str = "NetClient";

pub struct NetClientBuilder {
    builder: reqwest::ClientBuilder,
}

impl NetClientBuilder {
    pub fn new() -> NetClientBuilder {
        Self {
            builder: reqwest::ClientBuilder::new(),
        }
    }

    pub fn headers<K, V>(mut self, headers: &[(K, V)]) -> LuaResult<Self>
    where
        K: AsRef<str>,
        V: AsRef<[u8]>,
    {
        let mut map = HeaderMap::new();
        for (key, val) in headers {
            let hkey = HeaderName::from_str(key.as_ref()).into_lua_err()?;
            let hval = HeaderValue::from_bytes(val.as_ref()).into_lua_err()?;
            map.insert(hkey, hval);
        }
        self.builder = self.builder.default_headers(map);
        Ok(self)
    }

    pub fn build(self) -> LuaResult<NetClient> {
        let client = self.builder.build().into_lua_err()?;
        Ok(NetClient { inner: client })
    }
}

#[derive(Debug, Clone)]
pub struct NetClient {
    inner: reqwest::Client,
}

impl NetClient {
    pub fn from_registry(lua: &Lua) -> Self {
        lua.named_registry_value(REGISTRY_KEY)
            .expect("Failed to get NetClient from lua registry")
    }

    pub fn into_registry(self, lua: &Lua) {
        lua.set_named_registry_value(REGISTRY_KEY, self)
            .expect("Failed to store NetClient in lua registry");
    }

    pub async fn request(&self, config: RequestConfig) -> LuaResult<NetClientResponse> {
        // Create and send the request
        let mut request = self.inner.request(config.method, config.url);
        for (query, values) in config.query {
            request = request.query(
                &values
                    .iter()
                    .map(|v| (query.as_str(), v))
                    .collect::<Vec<_>>(),
            );
        }
        for (header, values) in config.headers {
            for value in values {
                request = request.header(header.as_str(), value);
            }
        }
        let res = request
            .body(config.body.unwrap_or_default())
            .send()
            .await
            .into_lua_err()?;

        // Extract status, headers
        let res_status = res.status().as_u16();
        let res_status_text = res.status().canonical_reason();
        let res_headers = res.headers().clone();

        // Read response bytes
        let mut res_bytes = res.bytes().await.into_lua_err()?.to_vec();
        let mut res_decompressed = false;

        // Check for extra options, decompression
        if config.options.decompress {
            let decompress_format = res_headers
                .iter()
                .find(|(name, _)| {
                    name.as_str()
                        .eq_ignore_ascii_case(CONTENT_ENCODING.as_str())
                })
                .and_then(|(_, value)| value.to_str().ok())
                .and_then(CompressDecompressFormat::detect_from_header_str);
            if let Some(format) = decompress_format {
                res_bytes = decompress(res_bytes, format).await?;
                res_decompressed = true;
            }
        }

        Ok(NetClientResponse {
            ok: (200..300).contains(&res_status),
            status_code: res_status,
            status_message: res_status_text.unwrap_or_default().to_string(),
            headers: res_headers,
            body: res_bytes,
            body_decompressed: res_decompressed,
        })
    }
}

impl LuaUserData for NetClient {}

impl FromLua for NetClient {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        if let LuaValue::UserData(ud) = value {
            if let Ok(ctx) = ud.borrow::<NetClient>() {
                return Ok(ctx.clone());
            }
        }
        unreachable!("NetClient should only be used from registry")
    }
}

impl From<&Lua> for NetClient {
    fn from(value: &Lua) -> Self {
        value
            .named_registry_value(REGISTRY_KEY)
            .expect("Missing require context in lua registry")
    }
}

pub struct NetClientResponse {
    ok: bool,
    status_code: u16,
    status_message: String,
    headers: HeaderMap,
    body: Vec<u8>,
    body_decompressed: bool,
}

impl NetClientResponse {
    pub fn into_lua_table(self, lua: &Lua) -> LuaResult<LuaTable> {
        TableBuilder::new(lua.clone())?
            .with_value("ok", self.ok)?
            .with_value("statusCode", self.status_code)?
            .with_value("statusMessage", self.status_message)?
            .with_value(
                "headers",
                header_map_to_table(lua, self.headers, self.body_decompressed)?,
            )?
            .with_value("body", lua.create_string(&self.body)?)?
            .build_readonly()
    }
}
