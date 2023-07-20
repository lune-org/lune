use std::fmt;

use mlua::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum PromptKind {
    Text,
    Confirm,
    Select,
    MultiSelect,
}

impl PromptKind {
    fn get_all() -> Vec<Self> {
        vec![Self::Text, Self::Confirm, Self::Select, Self::MultiSelect]
    }
}

impl Default for PromptKind {
    fn default() -> Self {
        Self::Text
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
            /*
                If the user only typed the prompt kind slightly wrong, meaning
                it has some kind of space in it, a weird character, or an uppercase
                character, we should try to be permissive as possible and still work

                Not everyone is using an IDE with proper Luau type definitions
                installed, and Luau is still a permissive scripting language
                even though it has a strict (but optional) type system
            */
            let s = s
                .chars()
                .filter_map(|c| {
                    if c.is_ascii_alphabetic() {
                        Some(c.to_ascii_lowercase())
                    } else {
                        None
                    }
                })
                .collect::<String>();
            // If the prompt kind is still invalid we will
            // show the user a descriptive error message
            match s.as_ref() {
                "text" => Ok(Self::Text),
                "confirm" => Ok(Self::Confirm),
                "select" => Ok(Self::Select),
                "multiselect" => Ok(Self::MultiSelect),
                s => Err(LuaError::FromLuaConversionError {
                    from: "string",
                    to: "PromptKind",
                    message: Some(format!(
                        "Invalid prompt kind '{s}', valid kinds are:\n{}",
                        PromptKind::get_all()
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                }),
            }
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
            default_bool,
            default_string,
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
