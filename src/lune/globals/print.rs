use mlua::prelude::*;
use tokio::io::{self, AsyncWriteExt};

use crate::lune::{scheduler::LuaSchedulerExt, util::formatting::pretty_format_multi_value};

pub fn create(lua: &'static Lua) -> LuaResult<impl IntoLua<'_>> {
    lua.create_async_function(|_, args: LuaMultiValue| async move {
        let formatted = format!("{}\n", pretty_format_multi_value(&args)?);
        let mut stdout = io::stdout();
        stdout.write_all(formatted.as_bytes()).await?;
        stdout.flush().await?;
        Ok(())
    })
}
