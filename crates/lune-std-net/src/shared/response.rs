use http_body_util::Full;

use hyper::{
    body::{Bytes, Incoming},
    header::{HeaderName, HeaderValue},
    HeaderMap, Response as HyperResponse,
};

use mlua::prelude::*;

use crate::{
    server::config::ResponseConfig,
    shared::{headers::header_map_to_table, incoming::handle_incoming_body},
};

#[derive(Debug, Clone)]
pub struct Response {
    // NOTE: We use Bytes instead of Full<Bytes> to avoid
    // needing async when getting a reference to the body
    pub(crate) inner: HyperResponse<Bytes>,
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
            inner: HyperResponse::from_parts(parts, body),
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
        self.inner.body()
    }

    /**
        Clones the inner `hyper` response with its body
        type modified to `Full<Bytes>` for sending.
    */
    #[allow(dead_code)]
    pub fn as_full(&self) -> HyperResponse<Full<Bytes>> {
        let mut builder = HyperResponse::builder()
            .version(self.inner.version())
            .status(self.inner.status());

        builder
            .headers_mut()
            .expect("request was valid")
            .extend(self.inner.headers().clone());

        let body = Full::new(self.inner.body().clone());
        builder.body(body).expect("request was valid")
    }

    /**
        Takes the inner `hyper` response with its body
        type modified to `Full<Bytes>` for sending.
    */
    #[allow(dead_code)]
    pub fn into_full(self) -> HyperResponse<Full<Bytes>> {
        let (parts, body) = self.inner.into_parts();
        HyperResponse::from_parts(parts, Full::new(body))
    }
}

impl TryFrom<ResponseConfig> for Response {
    type Error = LuaError;
    fn try_from(config: ResponseConfig) -> Result<Self, Self::Error> {
        // 1. Create the inner response builder
        let mut builder = HyperResponse::builder().status(config.status);

        // 2. Append any headers passed as a table - builder
        //    headers may be None if builder is already invalid
        if let Some(headers) = builder.headers_mut() {
            for (key, values) in config.headers {
                let key = HeaderName::from_bytes(key.as_bytes()).into_lua_err()?;
                for value in values {
                    let value = HeaderValue::from_str(&value).into_lua_err()?;
                    headers.insert(key.clone(), value);
                }
            }
        }

        // 3. Convert response body bytes to the proper Body
        //    type that Hyper expects, if we got any bytes
        let body = config.body.map(Bytes::from).unwrap_or_default();

        // 4. Finally, attach the body, verifying that the response is valid
        Ok(Self {
            inner: builder.body(body).into_lua_err()?,
            decompressed: false,
        })
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
