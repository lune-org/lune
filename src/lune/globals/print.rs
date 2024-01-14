use std::io::Write as _;

use mlua::prelude::*;

use crate::lune::util::formatting::pretty_format_multi_value;

pub fn create(lua: &Lua) -> LuaResult<impl IntoLua<'_>> {
    lua.create_function(|_, args: LuaMultiValue| {
        let formatted = format!("{}\n", pretty_format_multi_value(&args)?);
        let mut stdout = std::io::stdout();
        stdout.write_all(formatted.as_bytes())?;
        stdout.flush()?;
        Ok(())
    })
}
