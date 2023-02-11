use std::path::{PathBuf, MAIN_SEPARATOR};

use mlua::prelude::*;
use tokio::fs;

use crate::utils::table::TableBuilder;

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_async_function("readFile", fs_read_file)?
        .with_async_function("readDir", fs_read_dir)?
        .with_async_function("writeFile", fs_write_file)?
        .with_async_function("writeDir", fs_write_dir)?
        .with_async_function("removeFile", fs_remove_file)?
        .with_async_function("removeDir", fs_remove_dir)?
        .with_async_function("isFile", fs_is_file)?
        .with_async_function("isDir", fs_is_dir)?
        .build_readonly()
}

async fn fs_read_file(_: &'static Lua, path: String) -> LuaResult<String> {
    fs::read_to_string(&path).await.map_err(LuaError::external)
}

async fn fs_read_dir(_: &'static Lua, path: String) -> LuaResult<Vec<String>> {
    let mut dir_strings = Vec::new();
    let mut dir = fs::read_dir(&path).await.map_err(LuaError::external)?;
    while let Some(dir_entry) = dir.next_entry().await.map_err(LuaError::external)? {
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
                .strip_prefix(&dir_string_prefix)
                .unwrap()
                .to_owned()
        })
        .collect::<Vec<_>>();
    Ok(dir_strings_no_prefix)
}

async fn fs_write_file(_: &'static Lua, (path, contents): (String, String)) -> LuaResult<()> {
    fs::write(&path, &contents)
        .await
        .map_err(LuaError::external)
}

async fn fs_write_dir(_: &'static Lua, path: String) -> LuaResult<()> {
    fs::create_dir_all(&path).await.map_err(LuaError::external)
}

async fn fs_remove_file(_: &'static Lua, path: String) -> LuaResult<()> {
    fs::remove_file(&path).await.map_err(LuaError::external)
}

async fn fs_remove_dir(_: &'static Lua, path: String) -> LuaResult<()> {
    fs::remove_dir_all(&path).await.map_err(LuaError::external)
}

async fn fs_is_file(_: &'static Lua, path: String) -> LuaResult<bool> {
    let path = PathBuf::from(path);
    if path.exists() {
        Ok(fs::metadata(path)
            .await
            .map_err(LuaError::external)?
            .is_file())
    } else {
        Ok(false)
    }
}

async fn fs_is_dir(_: &'static Lua, path: String) -> LuaResult<bool> {
    let path = PathBuf::from(path);
    if path.exists() {
        Ok(fs::metadata(path)
            .await
            .map_err(LuaError::external)?
            .is_dir())
    } else {
        Ok(false)
    }
}
