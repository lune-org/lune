use mlua::{Lua, MultiValue, Result, Table};

use crate::utils::formatting::{
    flush_stdout, pretty_format_multi_value, print_color, print_label, print_style,
};

pub fn new(lua: &Lua) -> Result<Table> {
    let tab = lua.create_table()?;
    tab.raw_set("resetColor", lua.create_function(console_reset_color)?)?;
    tab.raw_set("setColor", lua.create_function(console_set_color)?)?;
    tab.raw_set("resetStyle", lua.create_function(console_reset_style)?)?;
    tab.raw_set("setStyle", lua.create_function(console_set_style)?)?;
    tab.raw_set("format", lua.create_function(console_format)?)?;
    tab.raw_set("log", lua.create_function(console_log)?)?;
    tab.raw_set("info", lua.create_function(console_info)?)?;
    tab.raw_set("warn", lua.create_function(console_warn)?)?;
    tab.raw_set("error", lua.create_function(console_error)?)?;
    tab.set_readonly(true);
    Ok(tab)
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
