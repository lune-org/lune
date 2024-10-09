use mlua::prelude::*;
use std::path::{Component, Path, PathBuf};
use tokio::fs;

/**

tries these alternatives on given path if path doesn't exist

* .lua and .luau extension
* path.join("init.luau") and path.join("init.lua")

 */
pub async fn resolve_path(path: &Path) -> LuaResult<PathBuf> {
    let init_path = &path.join("init");

    for ext in ["lua", "luau"] {
        // try extension on given path
        let path = append_extension(path, ext);

        if fs::try_exists(&path).await? {
            return Ok(normalize_path(&path));
        };

        // try extension on given path's init
        let init_path = append_extension(init_path, ext);

        if fs::try_exists(&init_path).await? {
            return Ok(normalize_path(&init_path));
        };
    }

    Err(LuaError::runtime("Could not resolve path"))
}

/**

Removes useless components from the given path

### Example

`./path/./path` turns into `./path/path`

 */
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.clone().peek() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

/**

adds extension to path without replacing it's current extensions

### Example

appending `.luau` to `path/path.config` will return `path/path.config.luau`

 */
fn append_extension(path: impl Into<PathBuf>, ext: &'static str) -> PathBuf {
    let mut new: PathBuf = path.into();
    match new.extension() {
        // FUTURE: There's probably a better way to do this than converting to a lossy string
        Some(e) => new.set_extension(format!("{}.{ext}", e.to_string_lossy())),
        None => new.set_extension(ext),
    };
    new
}
