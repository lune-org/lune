use std::path::{PathBuf, MAIN_SEPARATOR};

use mlua::{Lua, Result, Table};
use tokio::fs;

pub fn new(lua: &Lua) -> Result<Table> {
    let tab = lua.create_table()?;
    tab.raw_set("readFile", lua.create_async_function(fs_read_file)?)?;
    tab.raw_set("readDir", lua.create_async_function(fs_read_dir)?)?;
    tab.raw_set("writeFile", lua.create_async_function(fs_write_file)?)?;
    tab.raw_set("writeDir", lua.create_async_function(fs_write_dir)?)?;
    tab.raw_set("removeFile", lua.create_async_function(fs_remove_file)?)?;
    tab.raw_set("removeDir", lua.create_async_function(fs_remove_dir)?)?;
    tab.raw_set("isFile", lua.create_async_function(fs_is_file)?)?;
    tab.raw_set("isDir", lua.create_async_function(fs_is_dir)?)?;
    tab.set_readonly(true);
    Ok(tab)
}

async fn fs_read_file(_: &Lua, path: String) -> Result<String> {
    fs::read_to_string(&path)
        .await
        .map_err(mlua::Error::external)
}

async fn fs_read_dir(_: &Lua, path: String) -> Result<Vec<String>> {
    let mut dir_strings = Vec::new();
    let mut dir = fs::read_dir(&path).await.map_err(mlua::Error::external)?;
    while let Some(dir_entry) = dir.next_entry().await.map_err(mlua::Error::external)? {
        if let Some(dir_path_str) = dir_entry.path().to_str() {
            dir_strings.push(dir_path_str.to_owned());
        } else {
            return Err(mlua::Error::RuntimeError(format!(
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

async fn fs_write_file(_: &Lua, (path, contents): (String, String)) -> Result<()> {
    fs::write(&path, &contents)
        .await
        .map_err(mlua::Error::external)
}

async fn fs_write_dir(_: &Lua, path: String) -> Result<()> {
    fs::create_dir_all(&path)
        .await
        .map_err(mlua::Error::external)
}

async fn fs_remove_file(_: &Lua, path: String) -> Result<()> {
    fs::remove_file(&path).await.map_err(mlua::Error::external)
}

async fn fs_remove_dir(_: &Lua, path: String) -> Result<()> {
    fs::remove_dir_all(&path)
        .await
        .map_err(mlua::Error::external)
}

async fn fs_is_file(_: &Lua, path: String) -> Result<bool> {
    let path = PathBuf::from(path);
    if path.exists() {
        Ok(fs::metadata(path)
            .await
            .map_err(mlua::Error::external)?
            .is_file())
    } else {
        Ok(false)
    }
}

async fn fs_is_dir(_: &Lua, path: String) -> Result<bool> {
    let path = PathBuf::from(path);
    if path.exists() {
        Ok(fs::metadata(path)
            .await
            .map_err(mlua::Error::external)?
            .is_dir())
    } else {
        Ok(false)
    }
}
