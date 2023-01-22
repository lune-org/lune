use mlua::{Lua, MultiValue, Result};

use crate::utils::{
    formatting::{flush_stdout, pretty_format_multi_value, print_color, print_label, print_style},
    table_builder::ReadonlyTableBuilder,
};

pub async fn create(lua: Lua) -> Result<Lua> {
    let print = |args: &MultiValue, throw: bool| -> Result<()> {
        let s = pretty_format_multi_value(args)?;
        if throw {
            eprintln!("{s}");
        } else {
            println!("{s}");
        }
        flush_stdout()?;
        Ok(())
    };
    lua.globals().raw_set(
        "console",
        ReadonlyTableBuilder::new(&lua)?
            .with_function("resetColor", |_, _: ()| print_color("reset"))?
            .with_function("setColor", |_, color: String| print_color(color))?
            .with_function("resetStyle", |_, _: ()| print_style("reset"))?
            .with_function("setStyle", |_, style: String| print_style(style))?
            .with_function("format", |_, args: MultiValue| {
                pretty_format_multi_value(&args)
            })?
            .with_function("log", move |_, args: MultiValue| print(&args, false))?
            .with_function("info", move |_, args: MultiValue| {
                print_label("info")?;
                print(&args, false)
            })?
            .with_function("warn", move |_, args: MultiValue| {
                print_label("warn")?;
                print(&args, false)
            })?
            .with_function("error", move |_, args: MultiValue| {
                print_label("error")?;
                print(&args, true)
            })?
            .build()?,
    )?;
    Ok(lua)
}
