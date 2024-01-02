use mlua::prelude::*;
use tokio::{
    io::{self, AsyncWriteExt},
    task,
};

use crate::lune::util::formatting::{format_label, pretty_format_multi_value};

pub fn create(lua: &Lua) -> LuaResult<impl IntoLua<'_>> {
    lua.create_function(|_, args: LuaMultiValue| {
        let formatted = format!(
            "{}\n{}\n",
            format_label("warn"),
            pretty_format_multi_value(&args)?
        );
        task::spawn(async move {
            let _res = async move {
                let mut stdout = io::stderr();
                stdout.write_all(formatted.as_bytes()).await?;
                stdout.flush().await?;
                Ok::<_, LuaError>(())
            };
            // FUTURE: Send any error back to scheduler and emit it properly
        });
        Ok(())
    })
}
