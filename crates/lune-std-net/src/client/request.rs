use bstr::BString;
use futures_lite::prelude::*;
use http_body_util::{BodyStream, Full};
use hyper::{
    body::{Bytes, Incoming},
    client::conn::http1::handshake,
    Method, Request as HyperRequest, Response as HyperResponse,
};

use mlua::prelude::*;

use crate::{
    client::stream::HttpRequestStream,
    shared::hyper::{HyperExecutor, HyperIo},
};

#[derive(Debug, Clone)]
pub struct Request {
    inner: HyperRequest<Full<Bytes>>,
}

impl Request {
    pub async fn send(self, lua: Lua) -> LuaResult<Response> {
        let stream = HttpRequestStream::connect(self.inner.uri()).await?;

        let (mut sender, conn) = handshake(HyperIo::from(stream))
            .await
            .map_err(LuaError::external)?;

        HyperExecutor::execute(lua, conn);

        let incoming = sender
            .send_request(self.inner)
            .await
            .map_err(LuaError::external)?;

        Response::from_incoming(incoming).await
    }
}

impl FromLua for Request {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        if let LuaValue::String(s) = value {
            // We got a string, assume it's a URL + GET method
            let uri = s.to_str()?;
            Ok(Self {
                inner: HyperRequest::builder()
                    .uri(uri.as_ref())
                    .body(Full::new(Bytes::new()))
                    .into_lua_err()?,
            })
        } else if let LuaValue::Table(t) = value {
            // URL is always required with table options
            let url = t.get::<String>("url")?;
            let builder = HyperRequest::builder().uri(url);

            // Add method, if provided
            let builder = match t.get::<Option<String>>("method") {
                Ok(Some(method)) => builder.method(method.as_str()),
                Ok(None) => builder.method(Method::GET),
                Err(e) => return Err(e),
            };

            // Add body, if provided
            let builder = match t.get::<Option<BString>>("body") {
                Ok(Some(body)) => builder.body(Full::new(body.to_vec().into())),
                Ok(None) => builder.body(Full::new(Bytes::new())),
                Err(e) => return Err(e),
            };

            Ok(Self {
                inner: builder.into_lua_err()?,
            })
        } else {
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: String::from("HttpRequest"),
                message: Some(String::from("HttpRequest must be a string or table")),
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    inner: HyperResponse<Full<Bytes>>,
}

impl Response {
    pub async fn from_incoming(incoming: HyperResponse<Incoming>) -> LuaResult<Self> {
        let (parts, body) = incoming.into_parts();

        let body = BodyStream::new(body)
            .try_fold(Vec::<u8>::new(), |mut body, chunk| {
                if let Some(chunk) = chunk.data_ref() {
                    body.extend_from_slice(chunk);
                }
                Ok(body)
            })
            .await
            .into_lua_err()?;

        let bytes = Full::new(Bytes::from(body));
        let inner = HyperResponse::from_parts(parts, bytes);

        Ok(Self { inner })
    }
}

impl LuaUserData for Response {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("ok", |_, this| Ok(this.inner.status().is_success()));
        fields.add_field_method_get("status", |_, this| Ok(this.inner.status().as_u16()));
    }
}
