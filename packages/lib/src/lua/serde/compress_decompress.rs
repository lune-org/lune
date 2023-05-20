use async_compression::tokio::write::{
    BrotliDecoder, BrotliEncoder, GzipDecoder, GzipEncoder, ZlibDecoder, ZlibEncoder,
};
use blocking::unblock;
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use mlua::prelude::*;
use tokio::io::AsyncWriteExt;

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
    let mut bytes = Vec::new();
    match format {
        CompressDecompressFormat::Brotli => {
            BrotliEncoder::new(&mut bytes)
                .write_all(source.as_ref())
                .await?
        }
        CompressDecompressFormat::GZip => {
            GzipEncoder::new(&mut bytes)
                .write_all(source.as_ref())
                .await?
        }
        CompressDecompressFormat::ZLib => {
            ZlibEncoder::new(&mut bytes)
                .write_all(source.as_ref())
                .await?
        }
        CompressDecompressFormat::LZ4 => {
            let source = source.as_ref().to_vec();
            bytes = unblock(move || compress_prepend_size(&source)).await;
        }
    }
    Ok(bytes)
}

pub async fn decompress<'lua>(
    format: CompressDecompressFormat,
    source: impl AsRef<[u8]>,
) -> LuaResult<Vec<u8>> {
    let mut bytes = Vec::new();
    match format {
        CompressDecompressFormat::Brotli => {
            BrotliDecoder::new(&mut bytes)
                .write_all(source.as_ref())
                .await?
        }
        CompressDecompressFormat::GZip => {
            GzipDecoder::new(&mut bytes)
                .write_all(source.as_ref())
                .await?
        }
        CompressDecompressFormat::ZLib => {
            ZlibDecoder::new(&mut bytes)
                .write_all(source.as_ref())
                .await?
        }
        CompressDecompressFormat::LZ4 => {
            let source = source.as_ref().to_vec();
            bytes = unblock(move || decompress_size_prepended(&source))
                .await
                .map_err(LuaError::external)?;
        }
    }
    Ok(bytes)
}
