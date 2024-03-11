use std::fmt::Write;

use console::{colors_enabled, set_colors_enabled, style, Style};
use mlua::prelude::*;
use once_cell::sync::Lazy;

const MAX_FORMAT_DEPTH: usize = 4;

const INDENT: &str = "    ";

pub const STYLE_RESET_STR: &str = "\x1b[0m";

// Colors
pub static COLOR_BLACK: Lazy<Style> = Lazy::new(|| Style::new().black());
pub static COLOR_RED: Lazy<Style> = Lazy::new(|| Style::new().red());
pub static COLOR_GREEN: Lazy<Style> = Lazy::new(|| Style::new().green());
pub static COLOR_YELLOW: Lazy<Style> = Lazy::new(|| Style::new().yellow());
pub static COLOR_BLUE: Lazy<Style> = Lazy::new(|| Style::new().blue());
pub static COLOR_PURPLE: Lazy<Style> = Lazy::new(|| Style::new().magenta());
pub static COLOR_CYAN: Lazy<Style> = Lazy::new(|| Style::new().cyan());
pub static COLOR_WHITE: Lazy<Style> = Lazy::new(|| Style::new().white());

// Styles
pub static STYLE_BOLD: Lazy<Style> = Lazy::new(|| Style::new().bold());
pub static STYLE_DIM: Lazy<Style> = Lazy::new(|| Style::new().dim());

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
        style("[").dim(),
        match s.as_ref().to_ascii_lowercase().as_str() {
            "info" => style("INFO").blue(),
            "warn" => style("WARN").yellow(),
            "error" => style("ERROR").red(),
            _ => style(""),
        },
        style("]").dim()
    )
}

pub fn format_style(style: Option<&'static Style>) -> String {
    if cfg!(test) {
        "".to_string()
    } else if let Some(style) = style {
        // HACK: We have no direct way of referencing the ansi color code
        // of the style that console::Style provides, and we also know for
        // sure that styles always include the reset sequence at the end,
        // unless we are in a CI environment on non-interactive terminal
        style
            .apply_to("")
            .to_string()
            .trim_end_matches(STYLE_RESET_STR)
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
    parent_table_addr: Option<String>,
    depth: usize,
) -> std::fmt::Result {
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
                    .replace('\r', r"\r")
                    .replace('\n', r"\n")
            )
        )?,
        LuaValue::Table(ref tab) => {
            let table_addr = Some(format!("{:p}", tab.to_pointer()));

            if depth >= MAX_FORMAT_DEPTH {
                write!(buffer, "{}", STYLE_DIM.apply_to("{ ... }"))?;
            } else if let Some(s) = call_table_tostring_metamethod(tab) {
                write!(buffer, "{s}")?;
            } else if depth >= 1 && parent_table_addr.eq(&table_addr) {
                write!(buffer, "{}", STYLE_DIM.apply_to("<self>"))?
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
                            pretty_format_value(buffer, &key, parent_table_addr.clone(), depth)?;
                            write!(buffer, "] {} ", STYLE_DIM.apply_to("="))?;
                        }
                    }
                    pretty_format_value(buffer, &value, parent_table_addr.clone(), depth + 1)?;
                    write!(buffer, "{}", STYLE_DIM.apply_to(","))?;
                    is_empty = false;
                }
                if is_empty {
                    write!(buffer, "{}", STYLE_DIM.apply_to(" }"))?;
                } else {
                    write!(buffer, "\n{depth_indent}{}", STYLE_DIM.apply_to("}"))?;
                }
            }
        }
        LuaValue::Vector(v) => write!(
            buffer,
            "{}",
            COLOR_PURPLE.apply_to(format!(
                "<vector({x}, {y}, {z})>",
                x = v.x(),
                y = v.y(),
                z = v.z()
            ))
        )?,
        LuaValue::Thread(_) => write!(buffer, "{}", COLOR_PURPLE.apply_to("<thread>"))?,
        LuaValue::Function(_) => write!(buffer, "{}", COLOR_PURPLE.apply_to("<function>"))?,
        LuaValue::UserData(u) => {
            if let Some(s) = call_userdata_tostring_metamethod(u) {
                write!(buffer, "{s}")?
            } else {
                write!(buffer, "{}", COLOR_PURPLE.apply_to("<userdata>"))?
            }
        }
        LuaValue::LightUserData(_) => write!(buffer, "{}", COLOR_PURPLE.apply_to("<userdata>"))?,
        LuaValue::Error(e) => write!(buffer, "{}", pretty_format_luau_error(e, false),)?,
    }
    Ok(())
}

pub fn pretty_format_multi_value(multi: &LuaMultiValue) -> LuaResult<String> {
    let mut buffer = String::new();
    let mut counter = 0;
    for value in multi {
        counter += 1;
        if let LuaValue::String(s) = value {
            write!(buffer, "{}", s.to_string_lossy()).into_lua_err()?;
        } else {
            let addr = format!("{:p}", value.to_pointer());

            pretty_format_value(&mut buffer, value, Some(addr), 0).into_lua_err()?;
        }
        if counter < multi.len() {
            write!(&mut buffer, " ").into_lua_err()?;
        }
    }
    Ok(buffer)
}

pub fn pretty_format_luau_error(e: &LuaError, colorized: bool) -> String {
    let previous_colors_enabled = if !colorized {
        set_colors_enabled(false);
        Some(colors_enabled())
    } else {
        None
    };
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
                    err_lines[index] = stack_begin.clone();
                    found_stack_begin = true;
                    break;
                }
            }
            // Add "Stack End" to the very end of the stack trace for symmetry
            if found_stack_begin {
                err_lines.push(stack_end.clone());
            }
            err_lines.join("\n")
        }
        LuaError::CallbackError { traceback, cause } => {
            // Find the best traceback (most lines) and the root error message
            // The traceback may also start with "override traceback:" which
            // means it was passed from somewhere that wants a custom trace,
            // so we should then respect that and get the best override instead
            let mut full_trace = traceback.to_string();
            let mut root_cause = cause.as_ref();
            let mut trace_override = false;
            while let LuaError::CallbackError { cause, traceback } = root_cause {
                let is_override = traceback.starts_with("override traceback:");
                if is_override {
                    if !trace_override || traceback.lines().count() > full_trace.len() {
                        full_trace = traceback
                            .trim_start_matches("override traceback:")
                            .to_string();
                        trace_override = true;
                    }
                } else if !trace_override {
                    full_trace = format!("{traceback}\n{full_trace}");
                }
                root_cause = cause;
            }
            // If we got a runtime error with an embedded traceback, we should
            // use that instead since it generally contains more information
            if matches!(root_cause, LuaError::RuntimeError(e) if e.contains("stack traceback:")) {
                pretty_format_luau_error(root_cause, colorized)
            } else {
                // Otherwise we format whatever root error we got using
                // the same error formatting as for above runtime errors
                format!(
                    "{}\n{}\n{}\n{}",
                    pretty_format_luau_error(root_cause, colorized),
                    stack_begin,
                    full_trace.trim_start_matches("stack traceback:\n"),
                    stack_end
                )
            }
        }
        LuaError::BadArgument { pos, cause, .. } => match cause.as_ref() {
            // TODO: Add more detail to this error message
            LuaError::FromLuaConversionError { from, to, .. } => {
                format!("Argument #{pos} must be of type '{to}', got '{from}'")
            }
            c => format!(
                "Bad argument #{pos}\n{}",
                pretty_format_luau_error(c, colorized)
            ),
        },
        e => format!("{e}"),
    };
    // Re-enable colors if they were previously enabled
    if let Some(true) = previous_colors_enabled {
        set_colors_enabled(true)
    }
    // Remove the script path from the error message
    // itself, it can be found in the stack trace
    let mut err_lines = err_string.lines().collect::<Vec<_>>();
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
    // Find where the stack trace stars and ends
    let stack_begin_idx =
        err_lines.iter().enumerate().find_map(
            |(i, line)| {
                if *line == stack_begin {
                    Some(i)
                } else {
                    None
                }
            },
        );
    let stack_end_idx =
        err_lines.iter().enumerate().find_map(
            |(i, line)| {
                if *line == stack_end {
                    Some(i)
                } else {
                    None
                }
            },
        );
    // If we have a stack trace, we should transform the formatting from the
    // default mlua formatting into something more friendly, similar to Roblox
    if let (Some(idx_start), Some(idx_end)) = (stack_begin_idx, stack_end_idx) {
        let stack_lines = err_lines
            .iter()
            .enumerate()
            // Filter out stack lines
            .filter_map(|(idx, line)| {
                if idx > idx_start && idx < idx_end {
                    Some(*line)
                } else {
                    None
                }
            })
            // Transform from mlua format into friendly format, while also
            // ensuring that leading whitespace / indentation is consistent
            .map(transform_stack_line)
            .collect::<Vec<_>>();
        fix_error_nitpicks(format!(
            "{}\n{}\n{}\n{}",
            err_lines
                .iter()
                .take(idx_start)
                .copied()
                .collect::<Vec<_>>()
                .join("\n"),
            stack_begin,
            stack_lines.join("\n"),
            stack_end,
        ))
    } else {
        fix_error_nitpicks(err_string)
    }
}

fn transform_stack_line(line: &str) -> String {
    match (line.find('['), line.find(']')) {
        (Some(idx_start), Some(idx_end)) => {
            let name = line[idx_start..idx_end + 1]
                .trim_start_matches('[')
                .trim_start_matches("string ")
                .trim_start_matches('"')
                .trim_end_matches(']')
                .trim_end_matches('"');
            let after_name = &line[idx_end + 1..];
            let line_num = match after_name.find(':') {
                Some(lineno_start) => match after_name[lineno_start + 1..].find(':') {
                    Some(lineno_end) => &after_name[lineno_start + 1..lineno_end + 1],
                    None => match after_name.contains("in function") || after_name.contains("in ?")
                    {
                        false => &after_name[lineno_start + 1..],
                        true => "",
                    },
                },
                None => "",
            };
            let func_name = match after_name.find("in function ") {
                Some(func_start) => after_name[func_start + 12..]
                    .trim()
                    .trim_end_matches('\'')
                    .trim_start_matches('\'')
                    .trim_start_matches("_G."),
                None => "",
            };
            let mut result = String::new();
            write!(
                result,
                "    Script '{}'",
                match name {
                    "C" => "[C]",
                    name => name,
                },
            )
            .unwrap();
            if !line_num.is_empty() {
                write!(result, ", Line {line_num}").unwrap();
            }
            if !func_name.is_empty() {
                write!(result, " - function {func_name}").unwrap();
            }
            result
        }
        (_, _) => line.to_string(),
    }
}

fn fix_error_nitpicks(full_message: String) -> String {
    full_message
        // Hacky fix for our custom require appearing as a normal script
        // TODO: It's probably better to pull in the regex crate here ..
        .replace("'require', Line 5", "'[C]' - function require")
        .replace("'require', Line 7", "'[C]' - function require")
        .replace("'require', Line 8", "'[C]' - function require")
        // Same thing here for our async script
        .replace("'async', Line 2", "'[C]'")
        .replace("'async', Line 3", "'[C]'")
        // Fix error calls in custom script chunks coming through
        .replace(
            "'[C]' - function error\n    Script '[C]' - function require",
            "'[C]' - function require",
        )
        // Fix strange double require
        .replace(
            "'[C]' - function require - function require",
            "'[C]' - function require",
        )
        // Fix strange double C
        .replace("'[C]'\n    Script '[C]'", "'[C]'")
}

fn call_table_tostring_metamethod<'a>(tab: &'a LuaTable<'a>) -> Option<String> {
    let f = match tab.get_metatable() {
        None => None,
        Some(meta) => match meta.get::<_, LuaFunction>(LuaMetaMethod::ToString.name()) {
            Ok(method) => Some(method),
            Err(_) => None,
        },
    }?;
    match f.call::<_, String>(()) {
        Ok(res) => Some(res),
        Err(_) => None,
    }
}

fn call_userdata_tostring_metamethod<'a>(tab: &'a LuaAnyUserData<'a>) -> Option<String> {
    let f = match tab.get_metatable() {
        Err(_) => None,
        Ok(meta) => match meta.get::<LuaFunction>(LuaMetaMethod::ToString.name()) {
            Ok(method) => Some(method),
            Err(_) => None,
        },
    }?;
    match f.call::<_, String>(()) {
        Ok(res) => Some(res),
        Err(_) => None,
    }
}
