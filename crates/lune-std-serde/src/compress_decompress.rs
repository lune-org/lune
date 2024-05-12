use std::io::{copy as copy_std, Cursor, Read as _, Write as _};

use mlua::prelude::*;

use lz4::{Decoder, EncoderBuilder};
use tokio::{
    io::{copy, BufReader},
    task::spawn_blocking,
};

use async_compression::{
    tokio::bufread::{
        BrotliDecoder, BrotliEncoder, GzipDecoder, GzipEncoder, ZlibDecoder, ZlibEncoder,
    },
    Level::Best as CompressionQuality,
};

/**
    A compression and decompression format supported by Lune.
*/
#[derive(Debug, Clone, Copy)]
pub enum CompressDecompressFormat {
    Brotli,
    GZip,
    LZ4,
    ZLib,
}

#[allow(dead_code)]
impl CompressDecompressFormat {
    /**
        Detects a supported compression format from the given bytes.
    */
    #[allow(clippy::missing_panics_doc)]
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

    /**
        Detects a supported compression format from the given header string.

        The given header script should be a valid `Content-Encoding` header value.
    */
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

/**
    Compresses the given bytes using the specified format.

    # Errors

    Errors when the compression fails.
*/
pub async fn compress<'lua>(
    source: impl AsRef<[u8]>,
    format: CompressDecompressFormat,
) -> LuaResult<Vec<u8>> {
    if let CompressDecompressFormat::LZ4 = format {
        let source = source.as_ref().to_vec();
        return spawn_blocking(move || compress_lz4(source))
            .await
            .into_lua_err()?
            .into_lua_err();
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

/**
    Decompresses the given bytes using the specified format.

    # Errors

    Errors when the decompression fails.
*/
pub async fn decompress<'lua>(
    source: impl AsRef<[u8]>,
    format: CompressDecompressFormat,
) -> LuaResult<Vec<u8>> {
    if let CompressDecompressFormat::LZ4 = format {
        let source = source.as_ref().to_vec();
        return spawn_blocking(move || decompress_lz4(source))
            .await
            .into_lua_err()?
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

// TODO: Remove the compatibility layer. Prepending size is no longer
// necessary, using lz4 create instead of lz4-flex, but we must remove
// it in a major version to not unexpectedly break compatibility

fn compress_lz4(input: Vec<u8>) -> LuaResult<Vec<u8>> {
    let mut input = Cursor::new(input);
    let mut output = Cursor::new(Vec::new());

    // Prepend size for compatibility with old lz4-flex implementation
    let len = input.get_ref().len() as u32;
    output.write_all(len.to_le_bytes().as_ref())?;

    let mut encoder = EncoderBuilder::new()
        .level(16)
        .checksum(lz4::ContentChecksum::ChecksumEnabled)
        .block_mode(lz4::BlockMode::Independent)
        .build(output)?;

    copy_std(&mut input, &mut encoder)?;
    let (output, result) = encoder.finish();
    result?;

    Ok(output.into_inner())
}

fn decompress_lz4(input: Vec<u8>) -> LuaResult<Vec<u8>> {
    let mut input = Cursor::new(input);

    // Skip size for compatibility with old lz4-flex implementation
    // Note that right now we use it for preallocating the output buffer
    // and a small efficiency gain, maybe we can expose this as some kind
    // of "size hint" parameter instead in the serde library in the future
    let mut size = [0; 4];
    input.read_exact(&mut size)?;

    let capacity = u32::from_le_bytes(size) as usize;
    let mut output = Cursor::new(Vec::with_capacity(capacity));

    let mut decoder = Decoder::new(input)?;
    copy_std(&mut decoder, &mut output)?;

    Ok(output.into_inner())
}
