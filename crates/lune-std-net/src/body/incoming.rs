use http_body_util::BodyExt;
use hyper::{
    HeaderMap,
    body::{Bytes, Incoming},
    header::CONTENT_ENCODING,
};

use mlua::prelude::*;

use lune_std_serde::{CompressDecompressFormat, decompress};

pub async fn handle_incoming_body(
    headers: &HeaderMap,
    body: Incoming,
    should_decompress: bool,
) -> LuaResult<(Bytes, bool)> {
    let mut body = body.collect().await.into_lua_err()?.to_bytes();

    let was_decompressed = if should_decompress {
        let decompress_format = headers
            .get(CONTENT_ENCODING)
            .and_then(|value| value.to_str().ok())
            .and_then(CompressDecompressFormat::detect_from_header_str);
        if let Some(format) = decompress_format {
            body = Bytes::from(decompress(body, format).await?);
            true
        } else {
            false
        }
    } else {
        false
    };

    Ok((body, was_decompressed))
}
