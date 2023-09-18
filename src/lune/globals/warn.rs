use mlua::prelude::*;
use tokio::io::{self, AsyncWriteExt};

use crate::lune::{
    scheduler::LuaSchedulerExt,
    util::formatting::{format_label, pretty_format_multi_value},
};

pub fn create(lua: &'static Lua) -> LuaResult<impl IntoLua<'_>> {
    lua.create_async_function(|_, args: LuaMultiValue| async move {
        let formatted = format!(
            "{}\n{}\n",
            format_label("warn"),
            pretty_format_multi_value(&args)?
        );
        let mut stdout = io::stderr();
        stdout.write_all(formatted.as_bytes()).await?;
        stdout.flush().await?;
        Ok(())
    })
}
