use mlua::prelude::*;
use std::path::{Path, PathBuf};
use tokio::fs;

/// tries these alternatives on given path:
///
/// * .lua and .luau extension
/// * path.join("init.luau") and path.join("init.lua")
pub async fn resolve_path(path: &Path) -> LuaResult<PathBuf> {
    let init_path = &path.join("init");

    for ext in ["lua", "luau"] {
        // try extension on given path
        let path = append_extension(path, ext);

        if fs::try_exists(&path).await? {
            return Ok(path);
        };

        // try extension on given path's init
        let init_path = append_extension(init_path, ext);

        if fs::try_exists(&init_path).await? {
            return Ok(init_path);
        };
    }

    Err(LuaError::runtime("Could not resolve path"))
}

fn append_extension(path: impl Into<PathBuf>, ext: &'static str) -> PathBuf {
    let mut new = path.into();
    match new.extension() {
        // FUTURE: There's probably a better way to do this than converting to a lossy string
        Some(e) => new.set_extension(format!("{}.{ext}", e.to_string_lossy())),
        None => new.set_extension(ext),
    };
    new
}
