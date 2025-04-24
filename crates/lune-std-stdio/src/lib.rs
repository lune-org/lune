#![allow(clippy::cargo_common_metadata)]

use std::{
    io::{stderr, stdin, stdout, Stdin},
    sync::{Arc, LazyLock},
};

use mlua::prelude::*;
use mlua_luau_scheduler::LuaSpawnExt;

use async_lock::Mutex as AsyncMutex;
use blocking::Unblock;
use futures_lite::{io::BufReader, prelude::*};

use lune_utils::{
    fmt::{pretty_format_multi_value, ValueFormatConfig},
    TableBuilder,
};

mod prompt;
mod style_and_color;

use self::prompt::{prompt, PromptOptions, PromptResult};
use self::style_and_color::{ColorKind, StyleKind};

const FORMAT_CONFIG: ValueFormatConfig = ValueFormatConfig::new()
    .with_max_depth(4)
    .with_colors_enabled(false);

static STDIN: LazyLock<Arc<AsyncMutex<BufReader<Unblock<Stdin>>>>> = LazyLock::new(|| {
    let stdin = Unblock::new(stdin());
    let reader = BufReader::new(stdin);
    Arc::new(AsyncMutex::new(reader))
});

/**
    Creates the `stdio` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("color", stdio_color)?
        .with_function("style", stdio_style)?
        .with_function("format", stdio_format)?
        .with_async_function("write", stdio_write)?
        .with_async_function("ewrite", stdio_ewrite)?
        .with_async_function("readLine", stdio_read_line)?
        .with_async_function("readToEnd", stdio_read_to_end)?
        .with_async_function("prompt", stdio_prompt)?
        .build_readonly()
}

fn stdio_color(lua: &Lua, color: ColorKind) -> LuaResult<LuaValue> {
    color.ansi_escape_sequence().into_lua(lua)
}

fn stdio_style(lua: &Lua, style: StyleKind) -> LuaResult<LuaValue> {
    style.ansi_escape_sequence().into_lua(lua)
}

fn stdio_format(_: &Lua, args: LuaMultiValue) -> LuaResult<String> {
    Ok(pretty_format_multi_value(&args, &FORMAT_CONFIG))
}

async fn stdio_write(_: Lua, s: LuaString) -> LuaResult<()> {
    let mut stdout = Unblock::new(stdout());
    stdout.write_all(&s.as_bytes()).await?;
    stdout.flush().await?;
    Ok(())
}

async fn stdio_ewrite(_: Lua, s: LuaString) -> LuaResult<()> {
    let mut stderr = Unblock::new(stderr());
    stderr.write_all(&s.as_bytes()).await?;
    stderr.flush().await?;
    Ok(())
}

async fn stdio_read_line(lua: Lua, (): ()) -> LuaResult<LuaString> {
    let mut string = String::new();
    let mut handle = STDIN.lock_arc().await;
    handle.read_line(&mut string).await?;
    lua.create_string(&string)
}

async fn stdio_read_to_end(lua: Lua, (): ()) -> LuaResult<LuaString> {
    let mut buffer = Vec::new();
    let mut handle = STDIN.lock_arc().await;
    handle.read_to_end(&mut buffer).await?;
    lua.create_string(&buffer)
}

async fn stdio_prompt(lua: Lua, options: PromptOptions) -> LuaResult<PromptResult> {
    lua.spawn_blocking(move || prompt(options))
        .await
        .into_lua_err()
}
