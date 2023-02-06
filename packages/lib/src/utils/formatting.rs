use std::fmt::Write;

use console::{style, Style};
use lazy_static::lazy_static;
use mlua::prelude::*;

const MAX_FORMAT_DEPTH: usize = 4;

const INDENT: &str = "    ";

pub const STYLE_RESET_STR: &str = "\x1b[0m";

lazy_static! {
    // Colors
    pub static ref COLOR_BLACK: Style = Style::new().black();
    pub static ref COLOR_RED: Style = Style::new().red();
    pub static ref COLOR_GREEN: Style = Style::new().green();
    pub static ref COLOR_YELLOW: Style = Style::new().yellow();
    pub static ref COLOR_BLUE: Style = Style::new().blue();
    pub static ref COLOR_PURPLE: Style = Style::new().magenta();
    pub static ref COLOR_CYAN: Style = Style::new().cyan();
    pub static ref COLOR_WHITE: Style = Style::new().white();
    // Styles
    pub static ref STYLE_BOLD: Style = Style::new().bold();
    pub static ref STYLE_DIM: Style = Style::new().dim();
}

fn can_be_plain_lua_table_key(s: &LuaString) -> bool {
    let str = s.to_string_lossy().to_string();
    let first_char = str.chars().next().unwrap();
    if first_char.is_alphabetic() {
        str.chars().all(|c| c == '_' || c.is_alphanumeric())
    } else {
        false
    }
}

pub fn format_label<S: AsRef<str>>(s: S) -> String {
    format!(
        "{}{}{} ",
        style("[").bold(),
        match s.as_ref().to_ascii_lowercase().as_str() {
            "info" => style("INFO").blue(),
            "warn" => style("WARN").yellow(),
            "error" => style("ERROR").red(),
            _ => style(""),
        },
        style("]").bold()
    )
}

pub fn format_style(style: Option<&'static Style>) -> String {
    if cfg!(test) {
        "".to_string()
    } else if let Some(style) = style {
        // HACK: We have no direct way of referencing the ansi color code
        // of the style that console::Style provides, and we also know for
        // sure that styles always include the reset sequence at the end
        style
            .apply_to("")
            .to_string()
            .strip_suffix(STYLE_RESET_STR)
            .unwrap()
            .to_string()
    } else {
        STYLE_RESET_STR.to_string()
    }
}

pub fn style_from_color_str<S: AsRef<str>>(s: S) -> LuaResult<Option<&'static Style>> {
    Ok(match s.as_ref() {
        "reset" => None,
        "black" => Some(&COLOR_BLACK),
        "red" => Some(&COLOR_RED),
        "green" => Some(&COLOR_GREEN),
        "yellow" => Some(&COLOR_YELLOW),
        "blue" => Some(&COLOR_BLUE),
        "purple" => Some(&COLOR_PURPLE),
        "cyan" => Some(&COLOR_CYAN),
        "white" => Some(&COLOR_WHITE),
        _ => {
            return Err(LuaError::RuntimeError(format!(
                "The color '{}' is not a valid color name",
                s.as_ref()
            )));
        }
    })
}

pub fn style_from_style_str<S: AsRef<str>>(s: S) -> LuaResult<Option<&'static Style>> {
    Ok(match s.as_ref() {
        "reset" => None,
        "bold" => Some(&STYLE_BOLD),
        "dim" => Some(&STYLE_DIM),
        _ => {
            return Err(LuaError::RuntimeError(format!(
                "The style '{}' is not a valid style name",
                s.as_ref()
            )));
        }
    })
}

pub fn pretty_format_value(
    buffer: &mut String,
    value: &LuaValue,
    depth: usize,
) -> std::fmt::Result {
    // TODO: Handle tables with cyclic references
    match &value {
        LuaValue::Nil => write!(buffer, "nil")?,
        LuaValue::Boolean(true) => write!(buffer, "{}", COLOR_YELLOW.apply_to("true"))?,
        LuaValue::Boolean(false) => write!(buffer, "{}", COLOR_YELLOW.apply_to("false"))?,
        LuaValue::Number(n) => write!(buffer, "{}", COLOR_CYAN.apply_to(format!("{n}")))?,
        LuaValue::Integer(i) => write!(buffer, "{}", COLOR_CYAN.apply_to(format!("{i}")))?,
        LuaValue::String(s) => write!(
            buffer,
            "\"{}\"",
            COLOR_GREEN.apply_to(
                s.to_string_lossy()
                    .replace('"', r#"\""#)
                    .replace('\r', r#"\r"#)
                    .replace('\n', r#"\n"#)
            )
        )?,
        LuaValue::Table(ref tab) => {
            if depth >= MAX_FORMAT_DEPTH {
                write!(buffer, "{}", STYLE_DIM.apply_to("{ ... }"))?;
            } else {
                let mut is_empty = false;
                let depth_indent = INDENT.repeat(depth);
                write!(buffer, "{}", STYLE_DIM.apply_to("{"))?;
                for pair in tab.clone().pairs::<LuaValue, LuaValue>() {
                    let (key, value) = pair.unwrap();
                    match &key {
                        LuaValue::String(s) if can_be_plain_lua_table_key(s) => write!(
                            buffer,
                            "\n{}{}{} {} ",
                            depth_indent,
                            INDENT,
                            s.to_string_lossy(),
                            STYLE_DIM.apply_to("=")
                        )?,
                        _ => {
                            write!(buffer, "\n{depth_indent}{INDENT}[")?;
                            pretty_format_value(buffer, &key, depth)?;
                            write!(buffer, "] {} ", STYLE_DIM.apply_to("="))?;
                        }
                    }
                    pretty_format_value(buffer, &value, depth + 1)?;
                    write!(buffer, "{}", STYLE_DIM.apply_to(","))?;
                    is_empty = false;
                }
                if is_empty {
                    write!(buffer, "{}", STYLE_DIM.apply_to(" }"))?;
                } else {
                    write!(buffer, "\n{}", STYLE_DIM.apply_to("}"))?;
                }
            }
        }
        LuaValue::Vector(x, y, z) => write!(
            buffer,
            "{}",
            COLOR_PURPLE.apply_to(format!("<vector({x}, {y}, {z})>"))
        )?,
        LuaValue::Thread(_) => write!(buffer, "{}", COLOR_PURPLE.apply_to("<thread>"))?,
        LuaValue::Function(_) => write!(buffer, "{}", COLOR_PURPLE.apply_to("<function>"))?,
        LuaValue::UserData(_) | LuaValue::LightUserData(_) => {
            write!(buffer, "{}", COLOR_PURPLE.apply_to("<userdata>"))?
        }
        _ => write!(buffer, "{}", STYLE_DIM.apply_to("?"))?,
    }
    Ok(())
}

pub fn pretty_format_multi_value(multi: &LuaMultiValue) -> LuaResult<String> {
    let mut buffer = String::new();
    let mut counter = 0;
    for value in multi {
        counter += 1;
        if let LuaValue::String(s) = value {
            write!(buffer, "{}", s.to_string_lossy()).map_err(LuaError::external)?;
        } else {
            pretty_format_value(&mut buffer, value, 0).map_err(LuaError::external)?;
        }
        if counter < multi.len() {
            write!(&mut buffer, " ").map_err(LuaError::external)?;
        }
    }
    Ok(buffer)
}

pub fn pretty_format_luau_error(e: &LuaError) -> String {
    let stack_begin = format!("[{}]", COLOR_BLUE.apply_to("Stack Begin"));
    let stack_end = format!("[{}]", COLOR_BLUE.apply_to("Stack End"));
    let err_string = match e {
        LuaError::RuntimeError(e) => {
            // Remove unnecessary prefix
            let mut err_string = e.to_string();
            if let Some(no_prefix) = err_string.strip_prefix("runtime error: ") {
                err_string = no_prefix.to_string();
            }
            // Add "Stack Begin" instead of default stack traceback string
            let mut err_lines = err_string
                .lines()
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            let mut found_stack_begin = false;
            for (index, line) in err_lines.clone().iter().enumerate().rev() {
                if *line == "stack traceback:" {
                    err_lines[index] = stack_begin;
                    found_stack_begin = true;
                    break;
                }
            }
            // Add "Stack End" to the very end of the stack trace for symmetry
            if found_stack_begin {
                err_lines.push(stack_end);
            }
            err_lines.join("\n")
        }
        LuaError::CallbackError { traceback, cause } => {
            // Find the best traceback (longest) and the root error message
            let mut best_trace = traceback;
            let mut root_cause = cause.as_ref();
            while let LuaError::CallbackError { cause, traceback } = root_cause {
                if traceback.len() > best_trace.len() {
                    best_trace = traceback;
                }
                root_cause = cause;
            }
            // Same error formatting as above
            format!(
                "{}\n{}\n{}\n{}",
                pretty_format_luau_error(root_cause),
                stack_begin,
                best_trace.strip_prefix("stack traceback:\n").unwrap(),
                stack_end
            )
        }
        LuaError::ToLuaConversionError { from, to, message } => {
            let msg = message
                .clone()
                .map_or_else(String::new, |m| format!("\nDetails:\n\t{m}"));
            format!(
                "Failed to convert Rust type '{}' into Luau type '{}'!{}",
                from, to, msg
            )
        }
        LuaError::FromLuaConversionError { from, to, message } => {
            let msg = message
                .clone()
                .map_or_else(String::new, |m| format!("\nDetails:\n\t{m}"));
            format!(
                "Failed to convert Luau type '{}' into Rust type '{}'!{}",
                from, to, msg
            )
        }
        e => format!("{e}"),
    };
    let mut err_lines = err_string.lines().collect::<Vec<_>>();
    // Remove the script path from the error message
    // itself, it can be found in the stack trace
    // FIXME: This no longer works now that we use
    // an exact name when our lune script is loaded
    if let Some(first_line) = err_lines.first() {
        if first_line.starts_with("[string \"") {
            if let Some(closing_bracket) = first_line.find("]:") {
                let after_closing_bracket = &first_line[closing_bracket + 2..first_line.len()];
                if let Some(last_colon) = after_closing_bracket.find(": ") {
                    err_lines[0] = &after_closing_bracket
                        [last_colon + 2..first_line.len() - closing_bracket - 2];
                } else {
                    err_lines[0] = after_closing_bracket
                }
            }
        }
    }
    // Merge all lines back together into one string
    err_lines.join("\n")
}
