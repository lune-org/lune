use hyper::{
    HeaderMap, Response as HyperResponse, StatusCode,
    body::Incoming,
    header::{CONTENT_TYPE, HeaderValue},
};

use mlua::prelude::*;

use crate::{
    body::{ReadableBody, handle_incoming_body},
    shared::{headers::header_map_to_table, lua::lua_table_to_header_map},
};

#[derive(Debug, Clone)]
pub struct Response {
    pub(crate) inner: HyperResponse<ReadableBody>,
    pub(crate) decompressed: bool,
}

impl Response {
    /**
        Creates a new response from a raw incoming response.
    */
    pub async fn from_incoming(
        incoming: HyperResponse<Incoming>,
        decompress: bool,
    ) -> LuaResult<Self> {
        let (parts, body) = incoming.into_parts();

        let (body, decompressed) = handle_incoming_body(&parts.headers, body, decompress).await?;

        Ok(Self {
            inner: HyperResponse::from_parts(parts, ReadableBody::from(body)),
            decompressed,
        })
    }

    /**
        Returns whether the request was successful or not.
    */
    pub fn status_ok(&self) -> bool {
        self.inner.status().is_success()
    }

    /**
        Returns the status code of the response.
    */
    pub fn status_code(&self) -> u16 {
        self.inner.status().as_u16()
    }

    /**
        Returns the status message of the response.
    */
    pub fn status_message(&self) -> &str {
        self.inner.status().canonical_reason().unwrap_or_default()
    }

    /**
        Returns the headers of the response.
    */
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    /**
        Returns the body of the response.
    */
    pub fn body(&self) -> &[u8] {
        self.inner.body().as_slice()
    }

    /**
        Clones the inner `hyper` response.
    */
    #[allow(dead_code)]
    pub fn clone_inner(&self) -> HyperResponse<ReadableBody> {
        self.inner.clone()
    }

    /**
        Takes the inner `hyper` response by ownership.
    */
    #[allow(dead_code)]
    pub fn into_inner(self) -> HyperResponse<ReadableBody> {
        self.inner
    }
}

impl FromLua for Response {
    fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        if let Ok(body) = ReadableBody::from_lua(value.clone(), lua) {
            // String or buffer is always a 200 text/plain response
            let mut response = HyperResponse::new(body);
            response
                .headers_mut()
                .insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
            Ok(Self {
                inner: response,
                decompressed: false,
            })
        } else if let LuaValue::Table(tab) = value {
            // Extract status (required)
            let status = tab.get::<u16>("status")?;
            let status = StatusCode::from_u16(status).into_lua_err()?;

            // Extract headers
            let headers = tab.get::<Option<LuaTable>>("headers")?;
            let headers = headers
                .map(|t| lua_table_to_header_map(&t))
                .transpose()?
                .unwrap_or_default();

            // Extract body
            let body = tab.get::<ReadableBody>("body")?;

            // Build the full response
            let mut response = HyperResponse::new(body);
            response.headers_mut().extend(headers);
            *response.status_mut() = status;

            // All good, validated and we got what we need
            Ok(Self {
                inner: response,
                decompressed: false,
            })
        } else {
            // Anything else is invalid
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "Response".to_string(),
                message: Some(format!(
                    "Invalid response - expected table/string/buffer, got {}",
                    value.type_name()
                )),
            })
        }
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
