use std::{
    fmt::Write as _,
    io::{self, Write as _},
};

use mlua::{MultiValue, Value};

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

pub fn flush_stdout() -> mlua::Result<()> {
    io::stdout().flush().map_err(mlua::Error::external)
}

fn can_be_plain_lua_table_key(s: &mlua::String) -> bool {
    let str = s.to_string_lossy().to_string();
    let first_char = str.chars().next().unwrap();
    if first_char.is_alphabetic() {
        str.chars().all(|c| c == '_' || c.is_alphanumeric())
    } else {
        false
    }
}

pub fn print_label<S: AsRef<str>>(s: S) -> mlua::Result<()> {
    print!(
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
    );
    flush_stdout()?;
    Ok(())
}

pub fn print_style<S: AsRef<str>>(s: S) -> mlua::Result<()> {
    print!(
        "{}",
        match s.as_ref() {
            "reset" => STYLE_RESET,
            "bold" => STYLE_BOLD,
            "dim" => STYLE_DIM,
            _ => {
                return Err(mlua::Error::RuntimeError(format!(
                    "The style '{}' is not a valid style name",
                    s.as_ref()
                )));
            }
        }
    );
    flush_stdout()?;
    Ok(())
}

pub fn print_color<S: AsRef<str>>(s: S) -> mlua::Result<()> {
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
                return Err(mlua::Error::RuntimeError(format!(
                    "The color '{}' is not a valid color name",
                    s.as_ref()
                )));
            }
        }
    );
    flush_stdout()?;
    Ok(())
}

pub fn pretty_format_value(buffer: &mut String, value: &Value, depth: usize) -> anyhow::Result<()> {
    // TODO: Handle tables with cyclic references
    // TODO: Handle other types like function, userdata, ...
    match &value {
        Value::Nil => write!(buffer, "nil")?,
        Value::Boolean(true) => write!(buffer, "{COLOR_YELLOW}true{COLOR_RESET}")?,
        Value::Boolean(false) => write!(buffer, "{COLOR_YELLOW}false{COLOR_RESET}")?,
        Value::Number(n) => write!(buffer, "{COLOR_CYAN}{n}{COLOR_RESET}")?,
        Value::Integer(i) => write!(buffer, "{COLOR_CYAN}{i}{COLOR_RESET}")?,
        Value::String(s) => write!(
            buffer,
            "{}\"{}\"{}",
            COLOR_GREEN,
            s.to_string_lossy()
                .replace('"', r#"\""#)
                .replace('\n', r#"\n"#),
            COLOR_RESET
        )?,
        Value::Table(ref tab) => {
            if depth >= MAX_FORMAT_DEPTH {
                write!(buffer, "{STYLE_DIM}{{ ... }}{STYLE_RESET}")?;
            } else {
                let mut is_empty = false;
                let depth_indent = INDENT.repeat(depth);
                write!(buffer, "{STYLE_DIM}{{{STYLE_RESET}")?;
                for pair in tab.clone().pairs::<Value, Value>() {
                    let (key, value) = pair?;
                    match &key {
                        Value::String(s) if can_be_plain_lua_table_key(s) => write!(
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
        Value::Vector(x, y, z) => {
            write!(buffer, "{COLOR_PURPLE}<vector({x}, {y}, {z})>{COLOR_RESET}",)?
        }
        Value::Thread(_) => write!(buffer, "{COLOR_PURPLE}<thread>{COLOR_RESET}")?,
        Value::Function(_) => write!(buffer, "{COLOR_PURPLE}<function>{COLOR_RESET}")?,
        Value::UserData(_) | Value::LightUserData(_) => {
            write!(buffer, "{COLOR_PURPLE}<userdata>{COLOR_RESET}")?
        }
        _ => write!(buffer, "?")?,
    }
    Ok(())
}

pub fn pretty_format_multi_value(multi: &MultiValue) -> mlua::Result<String> {
    let mut buffer = String::new();
    let mut counter = 0;
    for value in multi {
        counter += 1;
        if let Value::String(s) = value {
            write!(buffer, "{}", s.to_string_lossy()).map_err(mlua::Error::external)?;
        } else {
            pretty_format_value(&mut buffer, value, 0).map_err(mlua::Error::external)?;
        }
        if counter < multi.len() {
            write!(&mut buffer, " ").map_err(mlua::Error::external)?;
        }
    }
    Ok(buffer)
}

pub fn pretty_print_luau_error(e: &mlua::Error) {
    match e {
        mlua::Error::RuntimeError(e) => {
            eprintln!("{e}");
        }
        mlua::Error::CallbackError { cause, traceback } => {
            pretty_print_luau_error(cause.as_ref());
            eprintln!("Traceback:");
            eprintln!("{}", traceback.strip_prefix("stack traceback:\n").unwrap());
        }
        mlua::Error::ToLuaConversionError { from, to, message } => {
            let msg = message
                .clone()
                .map_or_else(String::new, |m| format!("\nDetails:\n\t{m}"));
            eprintln!(
                "Failed to convert Rust type '{}' into Luau type '{}'!{}",
                from, to, msg
            );
        }
        mlua::Error::FromLuaConversionError { from, to, message } => {
            let msg = message
                .clone()
                .map_or_else(String::new, |m| format!("\nDetails:\n\t{m}"));
            eprintln!(
                "Failed to convert Luau type '{}' into Rust type '{}'!{}",
                from, to, msg
            );
        }
        e => eprintln!("{e}"),
    }
}
