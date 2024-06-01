use std::collections::HashSet;
use std::fmt::{self, Write as _};

use mlua::prelude::*;

use super::{
    basic::{format_value_styled, lua_value_as_plain_string_key},
    config::ValueFormatConfig,
    style::STYLE_DIM,
};

const INDENT: &str = "    ";

/**
    Representation of a pointer in memory to a Lua value.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct LuaValueId(usize);

impl From<&LuaValue<'_>> for LuaValueId {
    fn from(value: &LuaValue<'_>) -> Self {
        Self(value.to_pointer() as usize)
    }
}

impl From<&LuaTable<'_>> for LuaValueId {
    fn from(table: &LuaTable) -> Self {
        Self(table.to_pointer() as usize)
    }
}

/**
    Formats the given value, recursively formatting tables
    up to the maximum depth specified in the config.

    NOTE: We return a result here but it's really just to make handling
    of the `write!` calls easier. Writing into a string should never fail.
*/
pub(crate) fn format_value_recursive(
    value: &LuaValue,
    config: &ValueFormatConfig,
    visited: &mut HashSet<LuaValueId>,
    depth: usize,
) -> Result<String, fmt::Error> {
    let mut buffer = String::new();

    if let LuaValue::Table(ref t) = value {
        if depth >= config.max_depth {
            write!(buffer, "{}", STYLE_DIM.apply_to("{ ... }"))?;
        } else if !visited.insert(LuaValueId::from(t)) {
            write!(buffer, "{}", STYLE_DIM.apply_to("{ recursive }"))?;
        } else {
            write!(buffer, "{}", STYLE_DIM.apply_to("{"))?;

            let values = t
                .clone()
                .pairs::<LuaValue, LuaValue>()
                .map(|res| res.expect("conversion to LuaValue should never fail"))
                .collect::<Vec<_>>();

            let is_empty = values.is_empty();
            let is_array = values
                .iter()
                .enumerate()
                .all(|(i, (key, _))| key.as_integer().is_some_and(|x| x == (i as i32) + 1));

            let formatted_values = if is_array {
                format_array(values, config, visited, depth)?
            } else {
                format_table(values, config, visited, depth)?
            };

            visited.remove(&LuaValueId::from(t));

            if is_empty {
                write!(buffer, " {}", STYLE_DIM.apply_to("}"))?;
            } else {
                write!(
                    buffer,
                    "\n{}\n{}{}",
                    formatted_values.join("\n"),
                    INDENT.repeat(depth),
                    STYLE_DIM.apply_to("}")
                )?;
            }
        }
    } else {
        let prefer_plain = depth == 0;
        write!(buffer, "{}", format_value_styled(value, prefer_plain))?;
    }

    Ok(buffer)
}

fn format_array(
    values: Vec<(LuaValue, LuaValue)>,
    config: &ValueFormatConfig,
    visited: &mut HashSet<LuaValueId>,
    depth: usize,
) -> Result<Vec<String>, fmt::Error> {
    values
        .into_iter()
        .map(|(_, value)| {
            Ok(format!(
                "{}{}{}",
                INDENT.repeat(1 + depth),
                format_value_recursive(&value, config, visited, depth + 1)?,
                STYLE_DIM.apply_to(","),
            ))
        })
        .collect()
}

fn format_table(
    values: Vec<(LuaValue, LuaValue)>,
    config: &ValueFormatConfig,
    visited: &mut HashSet<LuaValueId>,
    depth: usize,
) -> Result<Vec<String>, fmt::Error> {
    values
        .into_iter()
        .map(|(key, value)| {
            if let Some(plain_key) = lua_value_as_plain_string_key(&key) {
                Ok(format!(
                    "{}{plain_key} {} {}{}",
                    INDENT.repeat(1 + depth),
                    STYLE_DIM.apply_to("="),
                    format_value_recursive(&value, config, visited, depth + 1)?,
                    STYLE_DIM.apply_to(","),
                ))
            } else {
                Ok(format!(
                    "{}{}{}{} {} {}{}",
                    INDENT.repeat(1 + depth),
                    STYLE_DIM.apply_to("["),
                    format_value_recursive(&key, config, visited, depth + 1)?,
                    STYLE_DIM.apply_to("]"),
                    STYLE_DIM.apply_to("="),
                    format_value_recursive(&value, config, visited, depth + 1)?,
                    STYLE_DIM.apply_to(","),
                ))
            }
        })
        .collect()
}
