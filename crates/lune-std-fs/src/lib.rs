#![allow(clippy::cargo_common_metadata)]

use std::io::ErrorKind as IoErrorKind;
use std::path::PathBuf;

use async_fs as fs;
use bstr::{BString, ByteSlice};
use futures_lite::prelude::*;
use mlua::prelude::*;

use lune_utils::TableBuilder;

mod copy;
mod metadata;
mod options;

use self::copy::copy;
use self::metadata::FsMetadata;
use self::options::FsWriteOptions;

const TYPEDEFS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/types.d.luau"));

/**
    Returns a string containing type definitions for the `fs` standard library.
*/
#[must_use]
pub fn typedefs() -> String {
    TYPEDEFS.to_string()
}

/**
    Creates the `fs` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_async_function("readFile", fs_read_file)?
        .with_async_function("readDir", fs_read_dir)?
        .with_async_function("writeFile", fs_write_file)?
        .with_async_function("writeDir", fs_write_dir)?
        .with_async_function("removeFile", fs_remove_file)?
        .with_async_function("removeDir", fs_remove_dir)?
        .with_async_function("metadata", fs_metadata)?
        .with_async_function("isFile", fs_is_file)?
        .with_async_function("isDir", fs_is_dir)?
        .with_async_function("move", fs_move)?
        .with_async_function("copy", fs_copy)?
        .build_readonly()
}

async fn fs_read_file(lua: Lua, path: String) -> LuaResult<LuaString> {
    let bytes = fs::read(&path).await.into_lua_err()?;

    lua.create_string(bytes)
}

async fn fs_read_dir(_: Lua, path: String) -> LuaResult<Vec<String>> {
    let mut dir_strings = Vec::new();
    let mut dir = fs::read_dir(&path).await.into_lua_err()?;
    while let Some(dir_entry) = dir.try_next().await.into_lua_err()? {
        if let Some(dir_name_str) = dir_entry.file_name().to_str() {
            dir_strings.push(dir_name_str.to_owned());
        } else {
            return Err(LuaError::RuntimeError(format!(
                "File name could not be converted into a string: '{}'",
                dir_entry.file_name().to_string_lossy()
            )));
        }
    }
    Ok(dir_strings)
}

async fn fs_write_file(_: Lua, (path, contents): (String, BString)) -> LuaResult<()> {
    fs::write(&path, contents.as_bytes()).await.into_lua_err()
}

async fn fs_write_dir(_: Lua, path: String) -> LuaResult<()> {
    fs::create_dir_all(&path).await.into_lua_err()
}

async fn fs_remove_file(_: Lua, path: String) -> LuaResult<()> {
    fs::remove_file(&path).await.into_lua_err()
}

async fn fs_remove_dir(_: Lua, path: String) -> LuaResult<()> {
    fs::remove_dir_all(&path).await.into_lua_err()
}

async fn fs_metadata(_: Lua, path: String) -> LuaResult<FsMetadata> {
    match fs::metadata(path).await {
        Err(e) if e.kind() == IoErrorKind::NotFound => Ok(FsMetadata::not_found()),
        Ok(meta) => Ok(FsMetadata::from(meta)),
        Err(e) => Err(e.into()),
    }
}

async fn fs_is_file(_: Lua, path: String) -> LuaResult<bool> {
    match fs::metadata(path).await {
        Err(e) if e.kind() == IoErrorKind::NotFound => Ok(false),
        Ok(meta) => Ok(meta.is_file()),
        Err(e) => Err(e.into()),
    }
}

async fn fs_is_dir(_: Lua, path: String) -> LuaResult<bool> {
    match fs::metadata(path).await {
        Err(e) if e.kind() == IoErrorKind::NotFound => Ok(false),
        Ok(meta) => Ok(meta.is_dir()),
        Err(e) => Err(e.into()),
    }
}

async fn fs_move(_: Lua, (from, to, options): (String, String, FsWriteOptions)) -> LuaResult<()> {
    let path_from = PathBuf::from(from);
    if !path_from.exists() {
        return Err(LuaError::RuntimeError(format!(
            "No file or directory exists at the path '{}'",
            path_from.display()
        )));
    }
    let path_to = PathBuf::from(to);
    if !options.overwrite && path_to.exists() {
        return Err(LuaError::RuntimeError(format!(
            "A file or directory already exists at the path '{}'",
            path_to.display()
        )));
    }
    fs::rename(path_from, path_to).await.into_lua_err()?;
    Ok(())
}

async fn fs_copy(_: Lua, (from, to, options): (String, String, FsWriteOptions)) -> LuaResult<()> {
    copy(from, to, options).await
}
