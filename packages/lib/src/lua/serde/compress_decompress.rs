use async_compression::tokio::write::{
    BrotliDecoder, BrotliEncoder, GzipDecoder, GzipEncoder, ZlibDecoder, ZlibEncoder,
};
use mlua::prelude::*;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Copy)]
pub enum CompressDecompressFormat {
    Brotli,
    GZip,
    ZLib,
}

#[allow(dead_code)]
impl CompressDecompressFormat {
    pub fn detect_from_bytes(bytes: impl AsRef<[u8]>) -> Option<Self> {
        let bytes = bytes.as_ref();
        if bytes[0..4] == [0x0B, 0x24, 0x72, 0x68] {
            Some(Self::Brotli)
        } else if bytes[0..3] == [0x1F, 0x8B, 0x08] {
            Some(Self::GZip)
        }
        // https://stackoverflow.com/a/54915442
        else if (bytes[0..2] == [0x78, 0x01])
            || (bytes[0..2] == [0x78, 0x5E])
            || (bytes[0..2] == [0x78, 0x9C])
            || (bytes[0..2] == [0x78, 0xDA])
        {
            Some(Self::ZLib)
        } else {
            None
        }
    }

    pub fn detect_from_header_str(header: impl AsRef<str>) -> Option<Self> {
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding#directives
        match header.as_ref().to_ascii_lowercase().trim() {
            "br" => Some(Self::Brotli),
            "deflate" => Some(Self::ZLib),
            "gzip" => Some(Self::GZip),
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
                "zlib" => Ok(Self::ZLib),
                kind => Err(LuaError::FromLuaConversionError {
                    from: value.type_name(),
                    to: "CompressDecompressFormat",
                    message: Some(format!(
                        "Invalid format '{kind}', valid formats are:  brotli, gzip, zlib"
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
    }
    Ok(bytes)
}
