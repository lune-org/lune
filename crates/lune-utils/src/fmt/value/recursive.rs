use std::collections::HashSet;
use std::fmt::{self, Write as _};

use mlua::prelude::*;

use super::{
    basic::{format_value_styled, lua_value_as_plain_string_key},
    config::ValueFormatConfig,
    style::STYLE_DIM,
};

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
            writeln!(buffer, "{}", STYLE_DIM.apply_to("{"))?;

            for res in t.clone().pairs::<LuaValue, LuaValue>() {
                let (key, value) = res.expect("conversion to LuaValue should never fail");
                let formatted = if let Some(plain_key) = lua_value_as_plain_string_key(&key) {
                    format!(
                        "{plain_key} {} {}{}",
                        STYLE_DIM.apply_to("="),
                        format_value_recursive(&value, config, visited, depth + 1)?,
                        STYLE_DIM.apply_to(","),
                    )
                } else {
                    format!(
                        "{}{}{} {} {}{}",
                        STYLE_DIM.apply_to("["),
                        format_value_recursive(&key, config, visited, depth + 1)?,
                        STYLE_DIM.apply_to("]"),
                        STYLE_DIM.apply_to("="),
                        format_value_recursive(&value, config, visited, depth + 1)?,
                        STYLE_DIM.apply_to(","),
                    )
                };
                buffer.push_str(&formatted);
            }

            visited.remove(&LuaValueId::from(t));
            writeln!(buffer, "{}", STYLE_DIM.apply_to("}"))?;
        }
    } else {
        write!(buffer, "{}", format_value_styled(value))?;
    }

    Ok(buffer)
}
