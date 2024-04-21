use mlua::prelude::*;

use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use tokio::io::{copy, BufReader};

use async_compression::{
    tokio::bufread::{
        BrotliDecoder, BrotliEncoder, GzipDecoder, GzipEncoder, ZlibDecoder, ZlibEncoder,
    },
    Level::Best as CompressionQuality,
};

#[derive(Debug, Clone, Copy)]
pub enum CompressDecompressFormat {
    Brotli,
    GZip,
    LZ4,
    ZLib,
}

#[allow(dead_code)]
impl CompressDecompressFormat {
    pub fn detect_from_bytes(bytes: impl AsRef<[u8]>) -> Option<Self> {
        match bytes.as_ref() {
            // https://github.com/PSeitz/lz4_flex/blob/main/src/frame/header.rs#L28
            b if b.len() >= 4
                && matches!(
                    u32::from_le_bytes(b[0..4].try_into().unwrap()),
                    0x184D2204 | 0x184C2102
                ) =>
            {
                Some(Self::LZ4)
            }
            // https://github.com/dropbox/rust-brotli/blob/master/src/enc/brotli_bit_stream.rs#L2805
            b if b.len() >= 4
                && matches!(
                    b[0..3],
                    [0xE1, 0x97, 0x81] | [0xE1, 0x97, 0x82] | [0xE1, 0x97, 0x80]
                ) =>
            {
                Some(Self::Brotli)
            }
            // https://github.com/rust-lang/flate2-rs/blob/main/src/gz/mod.rs#L135
            b if b.len() >= 3 && matches!(b[0..3], [0x1F, 0x8B, 0x08]) => Some(Self::GZip),
            // https://stackoverflow.com/a/43170354
            b if b.len() >= 2
                && matches!(
                    b[0..2],
                    [0x78, 0x01] | [0x78, 0x5E] | [0x78, 0x9C] | [0x78, 0xDA]
                ) =>
            {
                Some(Self::ZLib)
            }
            _ => None,
        }
    }

    pub fn detect_from_header_str(header: impl AsRef<str>) -> Option<Self> {
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding#directives
        match header.as_ref().to_ascii_lowercase().trim() {
            "br" | "brotli" => Some(Self::Brotli),
            "deflate" => Some(Self::ZLib),
            "gz" | "gzip" => Some(Self::GZip),
            _ => None,
        }
    }
}

impl<'lua> FromLua<'lua> for CompressDecompressFormat {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        if let LuaValue::String(s) = &value {
            match s.to_string_lossy().to_ascii_lowercase().trim() {
                "brotli" => Ok(Self::Brotli),
                "gzip" => Ok(Self::GZip),
                "lz4" => Ok(Self::LZ4),
                "zlib" => Ok(Self::ZLib),
                kind => Err(LuaError::FromLuaConversionError {
                    from: value.type_name(),
                    to: "CompressDecompressFormat",
                    message: Some(format!(
                        "Invalid format '{kind}', valid formats are:  brotli, gzip, lz4, zlib"
                    )),
                }),
            }
        } else {
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "CompressDecompressFormat",
                message: None,
            })
        }
    }
}

pub async fn compress<'lua>(
    format: CompressDecompressFormat,
    source: impl AsRef<[u8]>,
) -> LuaResult<Vec<u8>> {
    if let CompressDecompressFormat::LZ4 = format {
        let source = source.as_ref().to_vec();
        return Ok(blocking::unblock(move || compress_prepend_size(&source)).await);
    }

    let mut bytes = Vec::new();
    let reader = BufReader::new(source.as_ref());

    match format {
        CompressDecompressFormat::Brotli => {
            let mut encoder = BrotliEncoder::with_quality(reader, CompressionQuality);
            copy(&mut encoder, &mut bytes).await?;
        }
        CompressDecompressFormat::GZip => {
            let mut encoder = GzipEncoder::with_quality(reader, CompressionQuality);
            copy(&mut encoder, &mut bytes).await?;
        }
        CompressDecompressFormat::ZLib => {
            let mut encoder = ZlibEncoder::with_quality(reader, CompressionQuality);
            copy(&mut encoder, &mut bytes).await?;
        }
        CompressDecompressFormat::LZ4 => unreachable!(),
    }

    Ok(bytes)
}

pub async fn decompress<'lua>(
    format: CompressDecompressFormat,
    source: impl AsRef<[u8]>,
) -> LuaResult<Vec<u8>> {
    if let CompressDecompressFormat::LZ4 = format {
        let source = source.as_ref().to_vec();
        return blocking::unblock(move || decompress_size_prepended(&source))
            .await
            .into_lua_err();
    }

    let mut bytes = Vec::new();
    let reader = BufReader::new(source.as_ref());

    match format {
        CompressDecompressFormat::Brotli => {
            let mut decoder = BrotliDecoder::new(reader);
            copy(&mut decoder, &mut bytes).await?;
        }
        CompressDecompressFormat::GZip => {
            let mut decoder = GzipDecoder::new(reader);
            copy(&mut decoder, &mut bytes).await?;
        }
        CompressDecompressFormat::ZLib => {
            let mut decoder = ZlibDecoder::new(reader);
            copy(&mut decoder, &mut bytes).await?;
        }
        CompressDecompressFormat::LZ4 => unreachable!(),
    }

    Ok(bytes)
}
