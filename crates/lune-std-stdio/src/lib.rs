#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;
use mlua_luau_scheduler::LuaSpawnExt;

use tokio::io::{stderr, stdin, stdout, AsyncReadExt, AsyncWriteExt};

use lune_utils::TableBuilder;

mod prompt;

use self::prompt::{prompt, PromptOptions, PromptResult};

/**
    Creates the `stdio` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("color", stdio_color)?
        .with_function("style", stdio_style)?
        .with_function("format", stdio_format)?
        .with_async_function("write", stdio_write)?
        .with_async_function("ewrite", stdio_ewrite)?
        .with_async_function("readToEnd", stdio_read_to_end)?
        .with_async_function("prompt", stdio_prompt)?
        .build_readonly()
}

fn stdio_color(_: &Lua, _color: String) -> LuaResult<String> {
    // TODO: Migrate from old crate
    unimplemented!()
}

fn stdio_style(_: &Lua, _color: String) -> LuaResult<String> {
    // TODO: Migrate from old crate
    unimplemented!()
}

fn stdio_format(_: &Lua, _args: LuaMultiValue) -> LuaResult<String> {
    // TODO: Migrate from old crate
    unimplemented!()
}

async fn stdio_write(_: &Lua, s: LuaString<'_>) -> LuaResult<()> {
    let mut stdout = stdout();
    stdout.write_all(s.as_bytes()).await?;
    stdout.flush().await?;
    Ok(())
}

async fn stdio_ewrite(_: &Lua, s: LuaString<'_>) -> LuaResult<()> {
    let mut stderr = stderr();
    stderr.write_all(s.as_bytes()).await?;
    stderr.flush().await?;
    Ok(())
}

/*
    FUTURE: Figure out how to expose some kind of "readLine" function using a buffered reader.

    This is a bit tricky since we would want to be able to use **both** readLine and readToEnd
    in the same script, doing something like readLine, readLine, readToEnd from lua, and
    having that capture the first two lines and then read the rest of the input.
*/

async fn stdio_read_to_end(lua: &Lua, (): ()) -> LuaResult<LuaString> {
    let mut input = Vec::new();
    let mut stdin = stdin();
    stdin.read_to_end(&mut input).await?;
    lua.create_string(&input)
}

async fn stdio_prompt(lua: &Lua, options: PromptOptions) -> LuaResult<PromptResult> {
    lua.spawn_blocking(move || prompt(options))
        .await
        .into_lua_err()
}
