use std::path::{Path, PathBuf};

use mlua::prelude::*;

use super::context::*;

pub(super) async fn require<'lua, 'ctx>(
    lua: &'lua Lua,
    ctx: &'ctx RequireContext,
    source: &str,
    path: &str,
) -> LuaResult<LuaMultiValue<'lua>>
where
    'lua: 'ctx,
{
    let (abs_path, rel_path) = ctx.resolve_paths(source, path)?;
    require_abs_rel(lua, ctx, abs_path, rel_path).await
}

pub(super) async fn require_abs_rel<'lua, 'ctx>(
    lua: &'lua Lua,
    ctx: &'ctx RequireContext,
    abs_path: PathBuf, // Absolute to filesystem
    rel_path: PathBuf, // Relative to CWD (for displaying)
) -> LuaResult<LuaMultiValue<'lua>>
where
    'lua: 'ctx,
{
    // 1. Try to require the exact path
    match require_inner(lua, ctx, &abs_path, &rel_path).await {
        Ok(res) => return Ok(res),
        Err(LuaError::SyntaxError {
            message,
            incomplete_input: _,
        }) => {
            return Err(LuaError::runtime(message));
        }
        Err(_) => {}
    }

    // 2. Try to require the path with an added "luau" extension
    // 3. Try to require the path with an added "lua" extension
    for extension in ["luau", "lua"] {
        match require_inner(
            lua,
            ctx,
            &append_extension(&abs_path, extension),
            &append_extension(&rel_path, extension),
        )
        .await
        {
            Ok(res) => return Ok(res),
            Err(LuaError::SyntaxError {
                message,
                incomplete_input: _,
            }) => {
                return Err(LuaError::runtime(message));
            }
            Err(_) => {}
        }
    }

    // We didn't find any direct file paths, look
    // for directories with "init" files in them...
    let abs_init = abs_path.join("init");
    let rel_init = rel_path.join("init");

    // 4. Try to require the init path with an added "luau" extension
    // 5. Try to require the init path with an added "lua" extension
    for extension in ["luau", "lua"] {
        match require_inner(
            lua,
            ctx,
            &append_extension(&abs_init, extension),
            &append_extension(&rel_init, extension),
        )
        .await
        {
            Ok(res) => return Ok(res),
            Err(LuaError::SyntaxError {
                message,
                incomplete_input: _,
            }) => {
                return Err(LuaError::runtime(message));
            }
            Err(_) => {}
        }
    }

    // Nothing left to try, throw an error
    Err(LuaError::runtime(format!(
        "No file exists at the path '{}'",
        rel_path.display()
    )))
}

async fn require_inner<'lua, 'ctx>(
    lua: &'lua Lua,
    ctx: &'ctx RequireContext,
    abs_path: impl AsRef<Path>,
    rel_path: impl AsRef<Path>,
) -> LuaResult<LuaMultiValue<'lua>>
where
    'lua: 'ctx,
{
    let abs_path = abs_path.as_ref();
    let rel_path = rel_path.as_ref();

    if ctx.is_cached(abs_path)? {
        ctx.get_from_cache(lua, abs_path)
    } else if ctx.is_pending(abs_path)? {
        ctx.wait_for_cache(lua, &abs_path).await
    } else {
        ctx.load_with_caching(lua, &abs_path, &rel_path).await
    }
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
