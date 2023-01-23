use mlua::prelude::*;

use crate::utils::{
    formatting::{flush_stdout, pretty_format_multi_value, print_color, print_label, print_style},
    table_builder::TableBuilder,
};

pub async fn create(lua: &Lua) -> LuaResult<()> {
    let print = |args: &LuaMultiValue, throw: bool| -> LuaResult<()> {
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
        TableBuilder::new(lua)?
            .with_function("resetColor", |_, _: ()| print_color("reset"))?
            .with_function("setColor", |_, color: String| print_color(color))?
            .with_function("resetStyle", |_, _: ()| print_style("reset"))?
            .with_function("setStyle", |_, style: String| print_style(style))?
            .with_function("format", |_, args: LuaMultiValue| {
                pretty_format_multi_value(&args)
            })?
            .with_function("log", move |_, args: LuaMultiValue| print(&args, false))?
            .with_function("info", move |_, args: LuaMultiValue| {
                print_label("info")?;
                print(&args, false)
            })?
            .with_function("warn", move |_, args: LuaMultiValue| {
                print_label("warn")?;
                print(&args, false)
            })?
            .with_function("error", move |_, args: LuaMultiValue| {
                print_label("error")?;
                print(&args, true)
            })?
            .build_readonly()?,
    )
}
