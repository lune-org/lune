use std::path::{Path, PathBuf};

use mlua::prelude::*;

use super::context::*;

pub(super) async fn require<'lua, 'ctx>(
    ctx: &'ctx RequireContext<'lua>,
    source: &str,
    path: &str,
) -> LuaResult<LuaMultiValue<'lua>>
where
    'lua: 'ctx,
{
    let (abs_path, rel_path) = ctx.resolve_paths(source, path)?;

    // 1. Try to require the exact path
    println!("1. REQUIRE: Exact");
    if let Ok(res) = require_inner(ctx, &abs_path, &rel_path).await {
        return Ok(res);
    }

    // 2. Try to require the path with an added "luau" extension
    println!("2. REQUIRE: Luau extension");
    let (luau_abs_path, luau_rel_path) = (
        append_extension(&abs_path, "luau"),
        append_extension(&rel_path, "luau"),
    );
    if let Ok(res) = require_inner(ctx, &luau_abs_path, &luau_rel_path).await {
        return Ok(res);
    }

    // 2. Try to require the path with an added "lua" extension
    println!("3. REQUIRE: Lua extension");
    let (lua_abs_path, lua_rel_path) = (
        append_extension(&abs_path, "lua"),
        append_extension(&rel_path, "lua"),
    );
    if let Ok(res) = require_inner(ctx, &lua_abs_path, &lua_rel_path).await {
        return Ok(res);
    }

    // Nothing left to try, throw an error
    println!("4. REQUIRE: Error");
    Err(LuaError::runtime(format!(
        "No file exist at the path '{}'",
        rel_path.display()
    )))
}

async fn require_inner<'lua, 'ctx>(
    ctx: &'ctx RequireContext<'lua>,
    abs_path: impl AsRef<Path>,
    rel_path: impl AsRef<Path>,
) -> LuaResult<LuaMultiValue<'lua>>
where
    'lua: 'ctx,
{
    let abs_path = abs_path.as_ref();
    let rel_path = rel_path.as_ref();

    if ctx.is_cached(abs_path)? {
        println!("Found cached, fetching from cache");
        ctx.get_from_cache(abs_path)
    } else if ctx.is_pending(abs_path)? {
        println!("Found pending, waiting for cache");
        ctx.wait_for_cache(&abs_path).await
    } else {
        println!("No cached, loading new");
        ctx.load_with_caching(&abs_path, &rel_path).await
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
