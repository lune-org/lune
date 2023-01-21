use mlua::{Lua, MultiValue, Result, Table};

use crate::utils::{
    formatting::{flush_stdout, pretty_format_multi_value, print_color, print_label, print_style},
    table_builder::ReadonlyTableBuilder,
};

pub fn new(lua: &Lua) -> Result<Table> {
    ReadonlyTableBuilder::new(lua)?
        .with_function("resetColor", console_reset_color)?
        .with_function("setColor", console_set_color)?
        .with_function("resetStyle", console_reset_style)?
        .with_function("setStyle", console_set_style)?
        .with_function("format", console_format)?
        .with_function("log", console_log)?
        .with_function("info", console_info)?
        .with_function("warn", console_warn)?
        .with_function("error", console_error)?
        .build()
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
    println!("{s}");
    flush_stdout()?;
    Ok(())
}

fn console_info(_: &Lua, args: MultiValue) -> Result<()> {
    print_label("info")?;
    let s = pretty_format_multi_value(&args)?;
    println!("{s}");
    flush_stdout()?;
    Ok(())
}

fn console_warn(_: &Lua, args: MultiValue) -> Result<()> {
    print_label("warn")?;
    let s = pretty_format_multi_value(&args)?;
    println!("{s}");
    flush_stdout()?;
    Ok(())
}

fn console_error(_: &Lua, args: MultiValue) -> Result<()> {
    print_label("error")?;
    let s = pretty_format_multi_value(&args)?;
    eprintln!("{s}");
    flush_stdout()?;
    Ok(())
}
