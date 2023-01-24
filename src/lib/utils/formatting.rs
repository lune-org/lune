use std::{
    fmt::Write as _,
    io::{self, Write as _},
};

use mlua::prelude::*;

const MAX_FORMAT_DEPTH: usize = 4;

const INDENT: &str = "    ";

// TODO: Use some crate for this instead

pub const COLOR_RESET: &str = if cfg!(test) { "" } else { "\x1B[0m" };
pub const COLOR_BLACK: &str = if cfg!(test) { "" } else { "\x1B[30m" };
pub const COLOR_RED: &str = if cfg!(test) { "" } else { "\x1B[31m" };
pub const COLOR_GREEN: &str = if cfg!(test) { "" } else { "\x1B[32m" };
pub const COLOR_YELLOW: &str = if cfg!(test) { "" } else { "\x1B[33m" };
pub const COLOR_BLUE: &str = if cfg!(test) { "" } else { "\x1B[34m" };
pub const COLOR_PURPLE: &str = if cfg!(test) { "" } else { "\x1B[35m" };
pub const COLOR_CYAN: &str = if cfg!(test) { "" } else { "\x1B[36m" };
pub const COLOR_WHITE: &str = if cfg!(test) { "" } else { "\x1B[37m" };

pub const STYLE_RESET: &str = if cfg!(test) { "" } else { "\x1B[22m" };
pub const STYLE_BOLD: &str = if cfg!(test) { "" } else { "\x1B[1m" };
pub const STYLE_DIM: &str = if cfg!(test) { "" } else { "\x1B[2m" };

pub fn flush_stdout() -> LuaResult<()> {
    io::stdout().flush().map_err(LuaError::external)
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
        "{}[{}{}{}{}]{} ",
        STYLE_BOLD,
        match s.as_ref().to_ascii_lowercase().as_str() {
            "info" => COLOR_BLUE,
            "warn" => COLOR_YELLOW,
            "error" => COLOR_RED,
            _ => COLOR_WHITE,
        },
        s.as_ref().to_ascii_uppercase(),
        COLOR_RESET,
        STYLE_BOLD,
        STYLE_RESET
    )
}

pub fn print_label<S: AsRef<str>>(s: S) -> LuaResult<()> {
    print!("{}", format_label(s));
    flush_stdout()?;
    Ok(())
}

pub fn print_style<S: AsRef<str>>(s: S) -> LuaResult<()> {
    print!(
        "{}",
        match s.as_ref() {
            "reset" => STYLE_RESET,
            "bold" => STYLE_BOLD,
            "dim" => STYLE_DIM,
            _ => {
                return Err(LuaError::RuntimeError(format!(
                    "The style '{}' is not a valid style name",
                    s.as_ref()
                )));
            }
        }
    );
    flush_stdout()?;
    Ok(())
}

pub fn print_color<S: AsRef<str>>(s: S) -> LuaResult<()> {
    print!(
        "{}",
        match s.as_ref() {
            "reset" => COLOR_RESET,
            "black" => COLOR_BLACK,
            "red" => COLOR_RED,
            "green" => COLOR_GREEN,
            "yellow" => COLOR_YELLOW,
            "blue" => COLOR_BLUE,
            "purple" => COLOR_PURPLE,
            "cyan" => COLOR_CYAN,
            "white" => COLOR_WHITE,
            _ => {
                return Err(LuaError::RuntimeError(format!(
                    "The color '{}' is not a valid color name",
                    s.as_ref()
                )));
            }
        }
    );
    flush_stdout()?;
    Ok(())
}

pub fn pretty_format_value(
    buffer: &mut String,
    value: &LuaValue,
    depth: usize,
) -> anyhow::Result<()> {
    // TODO: Handle tables with cyclic references
    match &value {
        LuaValue::Nil => write!(buffer, "nil")?,
        LuaValue::Boolean(true) => write!(buffer, "{COLOR_YELLOW}true{COLOR_RESET}")?,
        LuaValue::Boolean(false) => write!(buffer, "{COLOR_YELLOW}false{COLOR_RESET}")?,
        LuaValue::Number(n) => write!(buffer, "{COLOR_CYAN}{n}{COLOR_RESET}")?,
        LuaValue::Integer(i) => write!(buffer, "{COLOR_CYAN}{i}{COLOR_RESET}")?,
        LuaValue::String(s) => write!(
            buffer,
            "{}\"{}\"{}",
            COLOR_GREEN,
            s.to_string_lossy()
                .replace('"', r#"\""#)
                .replace('\n', r#"\n"#),
            COLOR_RESET
        )?,
        LuaValue::Table(ref tab) => {
            if depth >= MAX_FORMAT_DEPTH {
                write!(buffer, "{STYLE_DIM}{{ ... }}{STYLE_RESET}")?;
            } else {
                let mut is_empty = false;
                let depth_indent = INDENT.repeat(depth);
                write!(buffer, "{STYLE_DIM}{{{STYLE_RESET}")?;
                for pair in tab.clone().pairs::<LuaValue, LuaValue>() {
                    let (key, value) = pair?;
                    match &key {
                        LuaValue::String(s) if can_be_plain_lua_table_key(s) => write!(
                            buffer,
                            "\n{}{}{} {}={} ",
                            depth_indent,
                            INDENT,
                            s.to_string_lossy(),
                            STYLE_DIM,
                            STYLE_RESET
                        )?,
                        _ => {
                            write!(buffer, "\n{depth_indent}{INDENT}[")?;
                            pretty_format_value(buffer, &key, depth)?;
                            write!(buffer, "] {STYLE_DIM}={STYLE_RESET} ")?;
                        }
                    }
                    pretty_format_value(buffer, &value, depth + 1)?;
                    write!(buffer, "{STYLE_DIM},{STYLE_RESET}")?;
                    is_empty = false;
                }
                if is_empty {
                    write!(buffer, " {STYLE_DIM}}}{STYLE_RESET}")?;
                } else {
                    write!(buffer, "\n{depth_indent}{STYLE_DIM}}}{STYLE_RESET}")?;
                }
            }
        }
        LuaValue::Vector(x, y, z) => {
            write!(buffer, "{COLOR_PURPLE}<vector({x}, {y}, {z})>{COLOR_RESET}",)?
        }
        LuaValue::Thread(_) => write!(buffer, "{COLOR_PURPLE}<thread>{COLOR_RESET}")?,
        LuaValue::Function(_) => write!(buffer, "{COLOR_PURPLE}<function>{COLOR_RESET}")?,
        LuaValue::UserData(_) | LuaValue::LightUserData(_) => {
            write!(buffer, "{COLOR_PURPLE}<userdata>{COLOR_RESET}")?
        }
        _ => write!(buffer, "?")?,
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
    let stack_begin = format!("[{}Stack Begin{}]", COLOR_BLUE, COLOR_RESET);
    let stack_end = format!("[{}Stack End{}]", COLOR_BLUE, COLOR_RESET);
    let err_string = match e {
        LuaError::RuntimeError(e) => {
            // Add "Stack Begin" instead of default stack traceback string
            let err_string = e.to_string();
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
