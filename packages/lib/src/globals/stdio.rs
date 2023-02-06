use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use mlua::prelude::*;

use crate::utils::{
    formatting::{
        format_style, pretty_format_multi_value, style_from_color_str, style_from_style_str,
    },
    table::TableBuilder,
};

pub fn create(lua: &Lua) -> LuaResult<()> {
    lua.globals().raw_set(
        "stdio",
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
            .with_function("write", |_, s: String| {
                print!("{s}");
                Ok(())
            })?
            .with_function("ewrite", |_, s: String| {
                eprint!("{s}");
                Ok(())
            })?
            .with_function("prompt", prompt)?
            .build_readonly()?,
    )
}

fn prompt_theme() -> ColorfulTheme {
    ColorfulTheme::default()
}

fn prompt<'a>(
    lua: &'a Lua,
    (kind, message, options): (Option<String>, Option<String>, LuaValue<'a>),
) -> LuaResult<LuaValue<'a>> {
    match kind.map(|k| k.trim().to_ascii_lowercase()).as_deref() {
        None | Some("text") => {
            let theme = prompt_theme();
            let mut prompt = Input::with_theme(&theme);
            if let Some(message) = message {
                prompt.with_prompt(message);
            };
            if let LuaValue::String(s) = options {
                let txt = String::from_lua(LuaValue::String(s), lua)?;
                prompt.with_initial_text(&txt);
            };
            let input: String = prompt.allow_empty(true).interact_text()?;
            Ok(LuaValue::String(lua.create_string(&input)?))
        }
        Some("confirm") => {
            if let Some(message) = message {
                let theme = prompt_theme();
                let mut prompt = Confirm::with_theme(&theme);
                if let LuaValue::Boolean(b) = options {
                    prompt.default(b);
                };
                let result = prompt.with_prompt(&message).interact()?;
                Ok(LuaValue::Boolean(result))
            } else {
                Err(LuaError::RuntimeError(
                    "Argument #2 missing or nil".to_string(),
                ))
            }
        }
        Some(s) if matches!(s, "select" | "multiselect") => {
            let options = match options {
                LuaValue::Table(t) => {
                    let v: Vec<String> = Vec::from_lua(LuaValue::Table(t), lua)?;
                    if v.len() < 2 {
                        return Err(LuaError::RuntimeError(
                            "Options table must contain at least 2 options".to_string(),
                        ));
                    }
                    v
                }
                LuaValue::Nil => {
                    return Err(LuaError::RuntimeError(
                        "Argument #3 missing or nil".to_string(),
                    ))
                }
                value => {
                    return Err(LuaError::RuntimeError(format!(
                        "Argument #3 must be a table, got '{}'",
                        value.type_name()
                    )))
                }
            };
            if let Some(message) = message {
                match s {
                    "select" => {
                        let chosen = Select::with_theme(&prompt_theme())
                            .with_prompt(&message)
                            .items(&options)
                            .interact_opt()?;
                        Ok(match chosen {
                            Some(idx) => LuaValue::Number((idx + 1) as f64),
                            None => LuaValue::Nil,
                        })
                    }
                    "multiselect" => {
                        let chosen = MultiSelect::with_theme(&prompt_theme())
                            .with_prompt(&message)
                            .items(&options)
                            .interact_opt()?;
                        Ok(match chosen {
                            Some(indices) => indices
                                .iter()
                                .map(|idx| (*idx + 1) as f64)
                                .collect::<Vec<_>>()
                                .to_lua(lua)?,
                            None => LuaValue::Nil,
                        })
                    }
                    _ => unreachable!(),
                }
            } else {
                match s {
                    "select" => {
                        let chosen = Select::new().items(&options).interact_opt()?;
                        Ok(match chosen {
                            Some(idx) => LuaValue::Number((idx + 1) as f64),
                            None => LuaValue::Nil,
                        })
                    }
                    "multiselect" => {
                        let chosen = MultiSelect::new().items(&options).interact_opt()?;
                        Ok(match chosen {
                            Some(indices) => indices
                                .iter()
                                .map(|idx| (*idx + 1) as f64)
                                .collect::<Vec<_>>()
                                .to_lua(lua)?,
                            None => LuaValue::Nil,
                        })
                    }
                    _ => unreachable!(),
                }
            }
        }
        Some(s) => Err(LuaError::RuntimeError(format!(
            "Invalid stdio prompt kind: '{s}'"
        ))),
    }
}
