use mlua::prelude::*;

use crate::utils::{
    formatting::{pretty_format_multi_value, print_color, print_reset, print_style},
    table::TableBuilder,
};

pub fn create(lua: &Lua) -> LuaResult<()> {
    lua.globals().raw_set(
        "console",
        TableBuilder::new(lua)?
            .with_function("resetStyle", |_, _: ()| print_reset())?
            .with_function(
                "setStyle",
                |_, (color, style): (Option<String>, Option<String>)| {
                    if let Some(color) = color {
                        print_color(color)?;
                    }
                    if let Some(style) = style {
                        print_style(style)?;
                    }
                    Ok(())
                },
            )?
            .with_function("format", |_, args: LuaMultiValue| {
                pretty_format_multi_value(&args)
            })?
            .build_readonly()?,
    )
}
