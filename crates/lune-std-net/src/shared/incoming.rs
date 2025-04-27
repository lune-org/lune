use futures_lite::prelude::*;
use http_body_util::BodyStream;
use hyper::{
    body::{Body as _, Bytes, Incoming},
    HeaderMap,
};

use mlua::prelude::*;

pub async fn handle_incoming_body(
    _headers: &HeaderMap,
    body: Incoming,
    _decompress: bool,
) -> LuaResult<(Bytes, bool)> {
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

    // TODO: Decompress the body if necessary

    Ok((Bytes::from(body), false))
}
