use std::{process::ExitStatus, sync::Weak, time::Duration};

use mlua::prelude::*;
use tokio::{io, process::Child, sync::mpsc::Sender, task::spawn, time::sleep};

use crate::utils::{futures::AsyncTeeWriter, message::LuneMessage};

pub async fn pipe_and_inherit_child_process_stdio(
    mut child: Child,
) -> LuaResult<(ExitStatus, Vec<u8>, Vec<u8>)> {
    let mut child_stdout = child.stdout.take().unwrap();
    let mut child_stderr = child.stderr.take().unwrap();

    let stdout_thread = spawn(async move {
        let mut stdout = io::stdout();
        let mut tee = AsyncTeeWriter::new(&mut stdout);

        io::copy(&mut child_stdout, &mut tee)
            .await
            .map_err(LuaError::external)?;

        Ok::<_, LuaError>(tee.into_vec())
    });

    let stderr_thread = spawn(async move {
        let mut stderr = io::stderr();
        let mut tee = AsyncTeeWriter::new(&mut stderr);

        io::copy(&mut child_stderr, &mut tee)
            .await
            .map_err(LuaError::external)?;

        Ok::<_, LuaError>(tee.into_vec())
    });

    let status = child.wait().await.expect("Child process failed to start");

    let stdout_buffer = stdout_thread.await.expect("Tee writer for stdout errored");
    let stderr_buffer = stderr_thread.await.expect("Tee writer for stderr errored");

    Ok::<_, LuaError>((status, stdout_buffer?, stderr_buffer?))
}

pub async fn exit_and_yield_forever(lua: &'static Lua, exit_code: Option<u8>) -> LuaResult<()> {
    let sender = lua
        .app_data_ref::<Weak<Sender<LuneMessage>>>()
        .unwrap()
        .upgrade()
        .unwrap();
    // Send an exit signal to the main thread, which
    // will try to exit safely and as soon as possible
    sender
        .send(LuneMessage::Exit(exit_code.unwrap_or(0)))
        .await
        .map_err(LuaError::external)?;
    // Make sure to block the rest of this thread indefinitely since
    // the main thread may not register the exit signal right away
    sleep(Duration::MAX).await;
    Ok(())
}
