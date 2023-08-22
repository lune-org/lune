use std::io::ErrorKind as IoErrorKind;
use std::path::{PathBuf, MAIN_SEPARATOR};

use mlua::prelude::*;
use tokio::fs;

use crate::lune::lua::{
    fs::{copy, FsMetadata, FsWriteOptions},
    table::TableBuilder,
};

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
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

async fn fs_read_file(lua: &Lua, path: String) -> LuaResult<LuaString> {
    let bytes = fs::read(&path).await.into_lua_err()?;
    lua.create_string(bytes)
}

async fn fs_read_dir(_: &Lua, path: String) -> LuaResult<Vec<String>> {
    let mut dir_strings = Vec::new();
    let mut dir = fs::read_dir(&path).await.into_lua_err()?;
    while let Some(dir_entry) = dir.next_entry().await.into_lua_err()? {
        if let Some(dir_path_str) = dir_entry.path().to_str() {
            dir_strings.push(dir_path_str.to_owned());
        } else {
            return Err(LuaError::RuntimeError(format!(
                "File path could not be converted into a string: '{}'",
                dir_entry.path().display()
            )));
        }
    }
    let mut dir_string_prefix = path;
    if !dir_string_prefix.ends_with(MAIN_SEPARATOR) {
        dir_string_prefix.push(MAIN_SEPARATOR);
    }
    let dir_strings_no_prefix = dir_strings
        .iter()
        .map(|inner_path| {
            inner_path
                .trim()
                .trim_start_matches(&dir_string_prefix)
                .to_owned()
        })
        .collect::<Vec<_>>();
    Ok(dir_strings_no_prefix)
}

async fn fs_write_file(_: &Lua, (path, contents): (String, LuaString<'_>)) -> LuaResult<()> {
    fs::write(&path, &contents.as_bytes()).await.into_lua_err()
}

async fn fs_write_dir(_: &Lua, path: String) -> LuaResult<()> {
    fs::create_dir_all(&path).await.into_lua_err()
}

async fn fs_remove_file(_: &Lua, path: String) -> LuaResult<()> {
    fs::remove_file(&path).await.into_lua_err()
}

async fn fs_remove_dir(_: &Lua, path: String) -> LuaResult<()> {
    fs::remove_dir_all(&path).await.into_lua_err()
}

async fn fs_metadata(_: &Lua, path: String) -> LuaResult<FsMetadata> {
    match fs::metadata(path).await {
        Err(e) if e.kind() == IoErrorKind::NotFound => Ok(FsMetadata::not_found()),
        Ok(meta) => Ok(FsMetadata::from(meta)),
        Err(e) => Err(e.into()),
    }
}

async fn fs_is_file(_: &Lua, path: String) -> LuaResult<bool> {
    match fs::metadata(path).await {
        Err(e) if e.kind() == IoErrorKind::NotFound => Ok(false),
        Ok(meta) => Ok(meta.is_file()),
        Err(e) => Err(e.into()),
    }
}

async fn fs_is_dir(_: &Lua, path: String) -> LuaResult<bool> {
    match fs::metadata(path).await {
        Err(e) if e.kind() == IoErrorKind::NotFound => Ok(false),
        Ok(meta) => Ok(meta.is_dir()),
        Err(e) => Err(e.into()),
    }
}

async fn fs_move(_: &Lua, (from, to, options): (String, String, FsWriteOptions)) -> LuaResult<()> {
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

async fn fs_copy(_: &Lua, (from, to, options): (String, String, FsWriteOptions)) -> LuaResult<()> {
    copy(from, to, options).await
}
