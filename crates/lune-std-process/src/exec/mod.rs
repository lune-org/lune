use async_process::Child;
use futures_lite::prelude::*;

use mlua::prelude::*;

use lune_utils::TableBuilder;

use super::options::ProcessSpawnOptionsStdioKind;

mod tee_writer;
mod wait_for_child;

use self::wait_for_child::wait_for_child;

pub async fn exec(
    lua: Lua,
    mut child: Child,
    stdin: Option<Vec<u8>>,
    stdout: ProcessSpawnOptionsStdioKind,
    stderr: ProcessSpawnOptionsStdioKind,
) -> LuaResult<LuaTable> {
    // Write to stdin before anything else - if we got it
    if let Some(stdin) = stdin {
        let mut child_stdin = child.stdin.take().unwrap();
        child_stdin.write_all(&stdin).await.into_lua_err()?;
    }

    let res = wait_for_child(child, stdout, stderr).await?;

    /*
        NOTE: If an exit code was not given by the child process,
        we default to 1 if it yielded any error output, otherwise 0

        An exit code may be missing if the process was terminated by
        some external signal, which is the only time we use this default
    */
    let code = res
        .status
        .code()
        .unwrap_or(i32::from(!res.stderr.is_empty()));

    // Construct and return a readonly lua table with results
    let stdout = lua.create_string(&res.stdout)?;
    let stderr = lua.create_string(&res.stderr)?;
    TableBuilder::new(lua)?
        .with_value("ok", code == 0)?
        .with_value("code", code)?
        .with_value("stdout", stdout)?
        .with_value("stderr", stderr)?
        .build_readonly()
}
