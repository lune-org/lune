use std::io::Write as _;

use mlua::prelude::*;

use crate::lune::util::formatting::{format_label, pretty_format_multi_value};

pub fn create(lua: &Lua) -> LuaResult<impl IntoLua<'_>> {
    lua.create_function(|_, args: LuaMultiValue| {
        let formatted = format!(
            "{}\n{}\n",
            format_label("warn"),
            pretty_format_multi_value(&args)?
        );
        let mut stderr = std::io::stderr();
        stderr.write_all(formatted.as_bytes())?;
        stderr.flush()?;
        Ok(())
    })
}
