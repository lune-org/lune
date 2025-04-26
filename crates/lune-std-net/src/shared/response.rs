use futures_lite::prelude::*;
use http_body_util::BodyStream;

use hyper::{
    body::{Body, Bytes, Incoming},
    HeaderMap, Response as HyperResponse,
};

use mlua::prelude::*;

use crate::shared::headers::header_map_to_table;

#[derive(Debug, Clone)]
pub struct Response {
    inner: HyperResponse<Bytes>,
    decompressed: bool,
}

impl Response {
    pub async fn from_incoming(
        incoming: HyperResponse<Incoming>,
        decompressed: bool,
    ) -> LuaResult<Self> {
        let (parts, body) = incoming.into_parts();

        let size = body.size_hint().lower() as usize;
        let buffer = Vec::<u8>::with_capacity(size);
        let body = BodyStream::new(body)
            .try_fold(buffer, |mut body, chunk| {
                if let Some(chunk) = chunk.data_ref() {
                    body.extend_from_slice(chunk);
                }
                Ok(body)
            })
            .await
            .into_lua_err()?;

        let bytes = Bytes::from(body);
        let inner = HyperResponse::from_parts(parts, bytes);

        Ok(Self {
            inner,
            decompressed,
        })
    }

    pub fn status_ok(&self) -> bool {
        self.inner.status().is_success()
    }

    pub fn status_code(&self) -> u16 {
        self.inner.status().as_u16()
    }

    pub fn status_message(&self) -> &str {
        self.inner.status().canonical_reason().unwrap_or_default()
    }

    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    pub fn body(&self) -> &[u8] {
        self.inner.body()
    }
}

impl LuaUserData for Response {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("ok", |_, this| Ok(this.status_ok()));
        fields.add_field_method_get("statusCode", |_, this| Ok(this.status_code()));
        fields.add_field_method_get("statusMessage", |lua, this| {
            lua.create_string(this.status_message())
        });
        fields.add_field_method_get("headers", |lua, this| {
            header_map_to_table(lua, this.headers().clone(), this.decompressed)
        });
        fields.add_field_method_get("body", |lua, this| lua.create_string(this.body()));
    }
}
