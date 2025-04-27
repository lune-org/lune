use futures_lite::prelude::*;
use http_body_util::BodyStream;
use hyper::{
    body::{Body as _, Bytes, Incoming},
    header::CONTENT_ENCODING,
    HeaderMap,
};

use mlua::prelude::*;

use lune_std_serde::{decompress, CompressDecompressFormat};

pub async fn handle_incoming_body(
    headers: &HeaderMap,
    body: Incoming,
    should_decompress: bool,
) -> LuaResult<(Bytes, bool)> {
    let size = body.size_hint().lower() as usize;
    let buffer = Vec::<u8>::with_capacity(size);

    let mut body = BodyStream::new(body)
        .try_fold(buffer, |mut body, chunk| {
            if let Some(chunk) = chunk.data_ref() {
                body.extend_from_slice(chunk);
            }
            Ok(body)
        })
        .await
        .into_lua_err()?;

    let was_decompressed = if should_decompress {
        let decompress_format = headers
            .iter()
            .find(|(name, _)| {
                name.as_str()
                    .eq_ignore_ascii_case(CONTENT_ENCODING.as_str())
            })
            .and_then(|(_, value)| value.to_str().ok())
            .and_then(CompressDecompressFormat::detect_from_header_str);
        if let Some(format) = decompress_format {
            body = decompress(body, format).await?;
            true
        } else {
            false
        }
    } else {
        false
    };

    Ok((Bytes::from(body), was_decompressed))
}
