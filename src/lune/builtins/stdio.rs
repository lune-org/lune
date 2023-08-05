use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use mlua::prelude::*;
use tokio::{
    io::{self, AsyncWriteExt},
    task,
};

use crate::lune::lua::{
    stdio::{
        formatting::{
            format_style, pretty_format_multi_value, style_from_color_str, style_from_style_str,
        },
        prompt::{PromptKind, PromptOptions, PromptResult},
    },
    table::TableBuilder,
};

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("color", |_, color: String| {
            let ansi_string = format_style(style_from_color_str(&color)?);
            Ok(ansi_string)
        })?
        .with_function("style", |_, style: String| {
            let ansi_string = format_style(style_from_style_str(&style)?);
            Ok(ansi_string)
        })?
        .with_function("format", |_, args: LuaMultiValue| {
            pretty_format_multi_value(&args)
        })?
        .with_async_function("write", |_, s: LuaString| async move {
            let mut stdout = io::stdout();
            stdout.write_all(s.as_bytes()).await?;
            stdout.flush().await?;
            Ok(())
        })?
        .with_async_function("ewrite", |_, s: LuaString| async move {
            let mut stderr = io::stderr();
            stderr.write_all(s.as_bytes()).await?;
            stderr.flush().await?;
            Ok(())
        })?
        .with_async_function("prompt", |_, options: PromptOptions| async move {
            task::spawn_blocking(move || prompt(options))
                .await
                .into_lua_err()?
        })?
        .build_readonly()
}

fn prompt_theme() -> ColorfulTheme {
    ColorfulTheme::default()
}

fn prompt(options: PromptOptions) -> LuaResult<PromptResult> {
    let theme = prompt_theme();
    match options.kind {
        PromptKind::Text => {
            let input: String = Input::with_theme(&theme)
                .allow_empty(true)
                .with_prompt(options.text.unwrap_or_default())
                .with_initial_text(options.default_string.unwrap_or_default())
                .interact_text()?;
            Ok(PromptResult::String(input))
        }
        PromptKind::Confirm => {
            let mut prompt = Confirm::with_theme(&theme);
            if let Some(b) = options.default_bool {
                prompt.default(b);
            };
            let result = prompt
                .with_prompt(&options.text.expect("Missing text in prompt options"))
                .interact()?;
            Ok(PromptResult::Boolean(result))
        }
        PromptKind::Select => {
            let chosen = Select::with_theme(&prompt_theme())
                .with_prompt(&options.text.unwrap_or_default())
                .items(&options.options.expect("Missing options in prompt options"))
                .interact_opt()?;
            Ok(match chosen {
                Some(idx) => PromptResult::Index(idx + 1),
                None => PromptResult::None,
            })
        }
        PromptKind::MultiSelect => {
            let chosen = MultiSelect::with_theme(&prompt_theme())
                .with_prompt(&options.text.unwrap_or_default())
                .items(&options.options.expect("Missing options in prompt options"))
                .interact_opt()?;
            Ok(match chosen {
                None => PromptResult::None,
                Some(indices) => {
                    PromptResult::Indices(indices.iter().map(|idx| *idx + 1).collect())
                }
            })
        }
    }
}
