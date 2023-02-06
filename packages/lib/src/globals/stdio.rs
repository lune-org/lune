use mlua::prelude::*;

use crate::utils::{
    formatting::{
        format_style, pretty_format_multi_value, style_from_color_str, style_from_style_str,
    },
    table::TableBuilder,
};

pub fn create(lua: &Lua) -> LuaResult<()> {
    lua.globals().raw_set(
        "stdio",
        TableBuilder::new(lua)?
            .with_function("color", |_, color: String| {
                let ansi_string = format_style(style_from_color_str(&color)?);
                Ok(ansi_string)
            })?
            .with_function("style", |_, style: String| {
                let ansi_string = format_style(style_from_style_str(&style)?);
                Ok(ansi_string)
            })?
            .with_function("format", |_, args: LuaMultiValue| {
                pretty_format_multi_value(&args)
            })?
            .with_function("write", |_, s: String| {
                print!("{s}");
                Ok(())
            })?
            .with_function("ewrite", |_, s: String| {
                eprint!("{s}");
                Ok(())
            })?
            .build_readonly()?,
    )
}
