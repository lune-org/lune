use std::fmt::{self, Write as _};

use console::{colors_enabled, set_colors_enabled};
use mlua::prelude::*;

mod basic;
mod config;
mod metamethods;
mod style;

pub use self::config::ValueFormatConfig;

use self::basic::{format_value_styled, lua_value_as_plain_string_key};
use self::style::STYLE_DIM;

// NOTE: We return a result here but it's really just to make handling
// of the `write!` calls easier. Writing into a string should never fail.
fn format_value_inner(
    value: &LuaValue,
    config: &ValueFormatConfig,
    depth: usize,
) -> Result<String, fmt::Error> {
    let mut buffer = String::new();

    // TODO: Rewrite this section to not be recursive and
    // keep track of any recursive references to tables.
    if let LuaValue::Table(ref t) = value {
        if depth >= config.max_depth {
            write!(buffer, "{}", STYLE_DIM.apply_to("{ ... }"))?;
        } else {
            writeln!(buffer, "{}", STYLE_DIM.apply_to("{"))?;

            for res in t.clone().pairs::<LuaValue, LuaValue>() {
                let (key, value) = res.expect("conversion to LuaValue should never fail");
                let formatted = if let Some(plain_key) = lua_value_as_plain_string_key(&key) {
                    format!(
                        "{} {} {}{}",
                        plain_key,
                        STYLE_DIM.apply_to("="),
                        format_value_inner(&value, config, depth + 1)?,
                        STYLE_DIM.apply_to(","),
                    )
                } else {
                    format!(
                        "{}{}{} {} {}{}",
                        STYLE_DIM.apply_to("["),
                        format_value_inner(&key, config, depth + 1)?,
                        STYLE_DIM.apply_to("]"),
                        STYLE_DIM.apply_to("="),
                        format_value_inner(&value, config, depth + 1)?,
                        STYLE_DIM.apply_to(","),
                    )
                };
                buffer.push_str(&formatted);
            }

            writeln!(buffer, "{}", STYLE_DIM.apply_to("}"))?;
        }
    } else {
        write!(buffer, "{}", format_value_styled(value))?;
    }

    Ok(buffer)
}

/**
    Formats a Lua value into a pretty string using the given config.
*/
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn pretty_format_value(value: &LuaValue, config: &ValueFormatConfig) -> String {
    let colors_were_enabled = colors_enabled();
    set_colors_enabled(config.colors_enabled);
    let res = format_value_inner(value, config, 0);
    set_colors_enabled(colors_were_enabled);
    res.expect("using fmt for writing into strings should never fail")
}

/**
    Formats a Lua multi-value into a pretty string using the given config.

    Each value will be separated by a space.
*/
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn pretty_format_multi_value(values: &LuaMultiValue, config: &ValueFormatConfig) -> String {
    values
        .into_iter()
        .map(|value| pretty_format_value(value, config))
        .collect::<Vec<_>>()
        .join(" ")
}
