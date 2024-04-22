use mlua::prelude::*;

use super::{
    metamethods::{call_table_tostring_metamethod, call_userdata_tostring_metamethod},
    style::{COLOR_CYAN, COLOR_GREEN, COLOR_MAGENTA, COLOR_YELLOW},
};

const STRING_REPLACEMENTS: &[(&str, &str)] =
    &[("\"", r#"\""#), ("\t", r"\t"), ("\r", r"\r"), ("\n", r"\n")];

/**
    Tries to return the given value as a plain string key.

    A plain string key must:

    - Start with an alphabetic character.
    - Only contain alphanumeric characters and underscores.
*/
pub(crate) fn lua_value_as_plain_string_key(value: &LuaValue) -> Option<String> {
    if let LuaValue::String(s) = value {
        if let Ok(s) = s.to_str() {
            let first_valid = s.chars().next().is_some_and(|c| c.is_ascii_alphabetic());
            let all_valid = s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            if first_valid && all_valid {
                return Some(s.to_string());
            }
        }
    }
    None
}

/**
    Formats a Lua value into a pretty string.

    This does not recursively format tables.
*/
pub(crate) fn format_value_styled(value: &LuaValue) -> String {
    match value {
        LuaValue::Nil => COLOR_YELLOW.apply_to("nil").to_string(),
        LuaValue::Boolean(true) => COLOR_YELLOW.apply_to("true").to_string(),
        LuaValue::Boolean(false) => COLOR_YELLOW.apply_to("false").to_string(),
        LuaValue::Number(n) => COLOR_CYAN.apply_to(n).to_string(),
        LuaValue::Integer(i) => COLOR_CYAN.apply_to(i).to_string(),
        LuaValue::String(s) => COLOR_GREEN
            .apply_to({
                let mut s = s.to_string_lossy().to_string();
                for (from, to) in STRING_REPLACEMENTS {
                    s = s.replace(from, to);
                }
                format!(r#""{s}""#)
            })
            .to_string(),
        LuaValue::Vector(_) => COLOR_MAGENTA.apply_to("<vector>").to_string(),
        LuaValue::Thread(_) => COLOR_MAGENTA.apply_to("<thread>").to_string(),
        LuaValue::Function(_) => COLOR_MAGENTA.apply_to("<function>").to_string(),
        LuaValue::LightUserData(_) => COLOR_MAGENTA.apply_to("<pointer>").to_string(),
        LuaValue::UserData(u) => {
            if let Some(s) = call_userdata_tostring_metamethod(u) {
                s
            } else {
                COLOR_MAGENTA.apply_to("<userdata>").to_string()
            }
        }
        LuaValue::Table(t) => {
            if let Some(s) = call_table_tostring_metamethod(t) {
                s
            } else {
                COLOR_MAGENTA.apply_to("<table>").to_string()
            }
        }
        _ => COLOR_MAGENTA.apply_to("<?>").to_string(),
    }
}
