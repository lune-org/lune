use std::{fmt, str::FromStr};

use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use mlua::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum PromptKind {
    Text,
    Confirm,
    Select,
    MultiSelect,
}

impl PromptKind {
    const ALL: [PromptKind; 4] = [Self::Text, Self::Confirm, Self::Select, Self::MultiSelect];
}

impl Default for PromptKind {
    fn default() -> Self {
        Self::Text
    }
}

impl FromStr for PromptKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "text" => Ok(Self::Text),
            "confirm" => Ok(Self::Confirm),
            "select" => Ok(Self::Select),
            "multiselect" => Ok(Self::MultiSelect),
            _ => Err(()),
        }
    }
}

impl fmt::Display for PromptKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Text => "Text",
                Self::Confirm => "Confirm",
                Self::Select => "Select",
                Self::MultiSelect => "MultiSelect",
            }
        )
    }
}

impl<'lua> FromLua<'lua> for PromptKind {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        if let LuaValue::Nil = value {
            Ok(Self::default())
        } else if let LuaValue::String(s) = value {
            let s = s.to_str()?;
            s.parse().map_err(|()| LuaError::FromLuaConversionError {
                from: "string",
                to: "PromptKind",
                message: Some(format!(
                    "Invalid prompt kind '{s}', valid kinds are:\n{}",
                    PromptKind::ALL
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            })
        } else {
            Err(LuaError::FromLuaConversionError {
                from: "nil",
                to: "PromptKind",
                message: None,
            })
        }
    }
}

pub struct PromptOptions {
    pub kind: PromptKind,
    pub text: Option<String>,
    pub default_string: Option<String>,
    pub default_bool: Option<bool>,
    pub options: Option<Vec<String>>,
}

impl<'lua> FromLuaMulti<'lua> for PromptOptions {
    fn from_lua_multi(mut values: LuaMultiValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        // Argument #1 - prompt kind (optional)
        let kind = values
            .pop_front()
            .map(|value| PromptKind::from_lua(value, lua))
            .transpose()?
            .unwrap_or_default();
        // Argument #2 - prompt text (optional)
        let text = values
            .pop_front()
            .map(|text| String::from_lua(text, lua))
            .transpose()?;
        // Argument #3 - default value / options,
        // this is different per each prompt kind
        let (default_bool, default_string, options) = match values.pop_front() {
            None => (None, None, None),
            Some(options) => match options {
                LuaValue::Nil => (None, None, None),
                LuaValue::Boolean(b) => (Some(b), None, None),
                LuaValue::String(s) => (
                    None,
                    Some(String::from_lua(LuaValue::String(s), lua)?),
                    None,
                ),
                LuaValue::Table(t) => (
                    None,
                    None,
                    Some(Vec::<String>::from_lua(LuaValue::Table(t), lua)?),
                ),
                value => {
                    return Err(LuaError::FromLuaConversionError {
                        from: value.type_name(),
                        to: "PromptOptions",
                        message: Some("Argument #3 must be a boolean, table, or nil".to_string()),
                    })
                }
            },
        };
        /*
            Make sure we got the required values for the specific prompt kind:

            - "Confirm" requires a message to be present so the user knows what they are confirming
            - "Select" and "MultiSelect" both require a table of options to choose from
        */
        if matches!(kind, PromptKind::Confirm) && text.is_none() {
            return Err(LuaError::FromLuaConversionError {
                from: "nil",
                to: "PromptOptions",
                message: Some("Argument #2 missing or nil".to_string()),
            });
        }
        if matches!(kind, PromptKind::Select | PromptKind::MultiSelect) && options.is_none() {
            return Err(LuaError::FromLuaConversionError {
                from: "nil",
                to: "PromptOptions",
                message: Some("Argument #3 missing or nil".to_string()),
            });
        }
        // All good, return the prompt options
        Ok(Self {
            kind,
            text,
            default_string,
            default_bool,
            options,
        })
    }
}

#[derive(Debug, Clone)]
pub enum PromptResult {
    String(String),
    Boolean(bool),
    Index(usize),
    Indices(Vec<usize>),
    None,
}

impl<'lua> IntoLua<'lua> for PromptResult {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(match self {
            Self::String(s) => LuaValue::String(lua.create_string(&s)?),
            Self::Boolean(b) => LuaValue::Boolean(b),
            Self::Index(i) => LuaValue::Number(i as f64),
            Self::Indices(v) => v.into_lua(lua)?,
            Self::None => LuaValue::Nil,
        })
    }
}

pub fn prompt(options: PromptOptions) -> LuaResult<PromptResult> {
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
            }
            let result = prompt
                .with_prompt(options.text.expect("Missing text in prompt options"))
                .interact()
                .into_lua_err()?;
            Ok(PromptResult::Boolean(result))
        }
        PromptKind::Select => {
            let chosen = Select::with_theme(&theme)
                .with_prompt(options.text.unwrap_or_default())
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
                .with_prompt(options.text.unwrap_or_default())
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
