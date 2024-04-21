use mlua::prelude::*;

use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use mlua_luau_scheduler::LuaSpawnExt;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

use crate::lune::util::{
    formatting::{
        format_style, pretty_format_multi_value, style_from_color_str, style_from_style_str,
    },
    TableBuilder,
};

mod prompt;
use prompt::{PromptKind, PromptOptions, PromptResult};

pub fn create(lua: &Lua) -> LuaResult<LuaTable<'_>> {
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

fn stdio_color(_: &Lua, color: String) -> LuaResult<String> {
    let ansi_string = format_style(style_from_color_str(&color)?);
    Ok(ansi_string)
}

fn stdio_style(_: &Lua, color: String) -> LuaResult<String> {
    let ansi_string = format_style(style_from_style_str(&color)?);
    Ok(ansi_string)
}

fn stdio_format(_: &Lua, args: LuaMultiValue) -> LuaResult<String> {
    pretty_format_multi_value(&args)
}

async fn stdio_write(_: &Lua, s: LuaString<'_>) -> LuaResult<()> {
    let mut stdout = io::stdout();
    stdout.write_all(s.as_bytes()).await?;
    stdout.flush().await?;
    Ok(())
}

async fn stdio_ewrite(_: &Lua, s: LuaString<'_>) -> LuaResult<()> {
    let mut stderr = io::stderr();
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

async fn stdio_read_to_end(lua: &Lua, _: ()) -> LuaResult<LuaString> {
    let mut input = Vec::new();
    let mut stdin = io::stdin();
    stdin.read_to_end(&mut input).await?;
    lua.create_string(&input)
}

async fn stdio_prompt(lua: &Lua, options: PromptOptions) -> LuaResult<PromptResult> {
    lua.spawn_blocking(move || prompt(options))
        .await
        .into_lua_err()
}

fn prompt(options: PromptOptions) -> LuaResult<PromptResult> {
    let theme = ColorfulTheme::default();
    match options.kind {
        PromptKind::Text => {
            let input: String = Input::with_theme(&theme)
                .allow_empty(true)
                .with_prompt(options.text.unwrap_or_default())
                .with_initial_text(options.default_string.unwrap_or_default())
                .interact_text()
                .into_lua_err()?;
            Ok(PromptResult::String(input))
        }
        PromptKind::Confirm => {
            let mut prompt = Confirm::with_theme(&theme);
            if let Some(b) = options.default_bool {
                prompt = prompt.default(b);
            };
            let result = prompt
                .with_prompt(&options.text.expect("Missing text in prompt options"))
                .interact()
                .into_lua_err()?;
            Ok(PromptResult::Boolean(result))
        }
        PromptKind::Select => {
            let chosen = Select::with_theme(&theme)
                .with_prompt(&options.text.unwrap_or_default())
                .items(&options.options.expect("Missing options in prompt options"))
                .interact_opt()
                .into_lua_err()?;
            Ok(match chosen {
                Some(idx) => PromptResult::Index(idx + 1),
                None => PromptResult::None,
            })
        }
        PromptKind::MultiSelect => {
            let chosen = MultiSelect::with_theme(&theme)
                .with_prompt(&options.text.unwrap_or_default())
                .items(&options.options.expect("Missing options in prompt options"))
                .interact_opt()
                .into_lua_err()?;
            Ok(match chosen {
                None => PromptResult::None,
                Some(indices) => {
                    PromptResult::Indices(indices.iter().map(|idx| *idx + 1).collect())
                }
            })
        }
    }
}
