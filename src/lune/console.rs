use std::{
    fmt::Write,
    io::{self, Write as IoWrite},
};

use mlua::{Lua, MultiValue, Result, UserData, UserDataMethods, Value};

const MAX_FORMAT_DEPTH: usize = 4;

const INDENT: &str = "    ";

const COLOR_RESET: &str = "\x1B[0m";
const COLOR_BLACK: &str = "\x1B[30m";
const COLOR_RED: &str = "\x1B[31m";
const COLOR_GREEN: &str = "\x1B[32m";
const COLOR_YELLOW: &str = "\x1B[33m";
const COLOR_BLUE: &str = "\x1B[34m";
const COLOR_PURPLE: &str = "\x1B[35m";
const COLOR_CYAN: &str = "\x1B[36m";
const COLOR_WHITE: &str = "\x1B[37m";

const STYLE_RESET: &str = "\x1B[22m";
const STYLE_BOLD: &str = "\x1B[1m";
const STYLE_DIM: &str = "\x1B[2m";

pub struct LuneConsole();

impl LuneConsole {
    pub fn new() -> Self {
        Self()
    }
}

impl UserData for LuneConsole {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("resetColor", console_reset_color);
        methods.add_function("setColor", console_set_color);
        methods.add_function("resetStyle", console_reset_style);
        methods.add_function("setStyle", console_set_style);
        methods.add_function("format", console_format);
        methods.add_function("log", console_log);
        methods.add_function("info", console_info);
        methods.add_function("warn", console_warn);
        methods.add_function("error", console_error);
    }
}

fn flush_stdout() -> Result<()> {
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

fn pretty_format_value(buffer: &mut String, value: &Value, depth: usize) -> anyhow::Result<()> {
    // TODO: Handle tables with cyclic references
    // TODO: Handle other types like function, userdata, ...
    match &value {
        Value::Nil => write!(buffer, "nil")?,
        Value::Boolean(true) => write!(buffer, "{}true{}", COLOR_YELLOW, COLOR_RESET)?,
        Value::Boolean(false) => write!(buffer, "{}false{}", COLOR_YELLOW, COLOR_RESET)?,
        Value::Number(n) => write!(buffer, "{}{}{}", COLOR_BLUE, n, COLOR_RESET)?,
        Value::Integer(i) => write!(buffer, "{}{}{}", COLOR_BLUE, i, COLOR_RESET)?,
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
                write!(buffer, "{}{{ ... }}{}", STYLE_DIM, STYLE_RESET)?;
            } else {
                let depth_indent = INDENT.repeat(depth);
                write!(buffer, "{}{{{}", STYLE_DIM, STYLE_RESET)?;
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
                            write!(buffer, "\n{}{}[", depth_indent, INDENT)?;
                            pretty_format_value(buffer, &key, depth)?;
                            write!(buffer, "] {}={} ", STYLE_DIM, STYLE_RESET)?;
                        }
                    }
                    pretty_format_value(buffer, &value, depth + 1)?;
                    write!(buffer, "{},{}", STYLE_DIM, STYLE_RESET)?;
                }
                write!(buffer, "\n{}{}}}{}", depth_indent, STYLE_DIM, STYLE_RESET)?;
            }
        }
        _ => write!(buffer, "?")?,
    }
    Ok(())
}

fn pretty_format_multi_value(multi: &MultiValue) -> Result<String> {
    let mut buffer = String::new();
    let mut counter = 0;
    for value in multi {
        counter += 1;
        if let Value::String(s) = value {
            write!(buffer, "{}", s.to_string_lossy()).map_err(mlua::Error::external)?
        } else {
            pretty_format_value(&mut buffer, value, 0).map_err(mlua::Error::external)?;
        }
        if counter < multi.len() {
            write!(&mut buffer, " ").map_err(mlua::Error::external)?;
        }
    }
    Ok(buffer)
}

fn print_style<S: AsRef<str>>(s: S) -> Result<()> {
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

fn print_color<S: AsRef<str>>(s: S) -> Result<()> {
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

fn console_reset_color(_: &Lua, _: ()) -> Result<()> {
    print_color("reset")?;
    flush_stdout()?;
    Ok(())
}

fn console_set_color(_: &Lua, color: String) -> Result<()> {
    print_color(color.trim().to_ascii_lowercase())?;
    Ok(())
}

fn console_reset_style(_: &Lua, _: ()) -> Result<()> {
    print_style("reset")?;
    flush_stdout()?;
    Ok(())
}

fn console_set_style(_: &Lua, style: String) -> Result<()> {
    print_style(style.trim().to_ascii_lowercase())?;
    Ok(())
}

fn console_format(_: &Lua, args: MultiValue) -> Result<String> {
    pretty_format_multi_value(&args)
}

fn console_log(_: &Lua, args: MultiValue) -> Result<()> {
    let s = pretty_format_multi_value(&args)?;
    println!("{}", s);
    flush_stdout()?;
    Ok(())
}

fn console_info(_: &Lua, args: MultiValue) -> Result<()> {
    print!(
        "{}{}[INFO]{}{} ",
        STYLE_BOLD, COLOR_CYAN, COLOR_RESET, STYLE_RESET
    );
    let s = pretty_format_multi_value(&args)?;
    println!("{}", s);
    flush_stdout()?;
    Ok(())
}

fn console_warn(_: &Lua, args: MultiValue) -> Result<()> {
    print!(
        "{}{}[WARN]{}{} ",
        STYLE_BOLD, COLOR_YELLOW, COLOR_RESET, STYLE_RESET
    );
    let s = pretty_format_multi_value(&args)?;
    println!("{}", s);
    flush_stdout()?;
    Ok(())
}

fn console_error(_: &Lua, args: MultiValue) -> Result<()> {
    eprint!(
        "{}{}[ERROR]{}{} ",
        STYLE_BOLD, COLOR_RED, COLOR_RESET, STYLE_RESET
    );
    let s = pretty_format_multi_value(&args)?;
    eprintln!("{}", s);
    flush_stdout()?;
    Ok(())
}
