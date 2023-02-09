// https://stackoverflow.com/questions/71141122/-

use std::{
    pin::Pin,
    process::ExitStatus,
    sync::Weak,
    task::{Context, Poll},
    time::Duration,
};

use mlua::prelude::*;
use tokio::{
    io::{self, AsyncWrite, AsyncWriteExt},
    process::Child,
    sync::mpsc::Sender,
    task, time,
};

use crate::LuneMessage;

pub struct TeeWriter<'a, L, R>
where
    L: AsyncWrite + Unpin,
    R: AsyncWrite + Unpin,
{
    left: &'a mut L,
    right: &'a mut R,
}

impl<'a, L, R> TeeWriter<'a, L, R>
where
    L: AsyncWrite + Unpin,
    R: AsyncWrite + Unpin,
{
    pub fn new(left: &'a mut L, right: &'a mut R) -> Self {
        Self { left, right }
    }
}

impl<'a, L, R> AsyncWrite for TeeWriter<'a, L, R>
where
    L: AsyncWrite + Unpin,
    R: AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // TODO: Figure out how to poll both of these
        // futures, we can't use await in this trait impl
        // It might be better to split the generic TeeWriter out
        // and instead make TeeStdoutWriter and TeeStderrWriter
        // structs that use Stdout and Stderr + Vec directly,
        // all of which already implement these traits for us
        self.left.write_all(buf);
        self.right.write_all(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        todo!()
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        todo!()
    }
}

pub async fn pipe_and_inherit_child_process_stdio(
    mut child: Child,
) -> LuaResult<(ExitStatus, Vec<u8>, Vec<u8>)> {
    // https://stackoverflow.com/questions/71141122/-
    let mut child_stdout = child.stdout.take().unwrap();
    let mut child_stderr = child.stderr.take().unwrap();

    // TODO: Or maybe we could just spawn four local tasks instead,
    // one for each vec and one each for Stdout and Stderr, then we
    // join the local tasks for the vecs to get out our results

    let stdout_thread = task::spawn_local(async move {
        let mut stdout = io::stdout();
        let mut log = Vec::new();
        let mut tee = TeeWriter::new(&mut stdout, &mut log);

        io::copy(&mut child_stdout, &mut tee)
            .await
            .map_err(LuaError::external)?;

        Ok(log)
    });

    let stderr_thread = task::spawn_local(async move {
        let mut stderr = io::stderr();
        let mut log = Vec::new();
        let mut tee = TeeWriter::new(&mut stderr, &mut log);

        io::copy(&mut child_stderr, &mut tee)
            .await
            .map_err(LuaError::external)?;

        Ok(log)
    });

    let status = child.wait().await.expect("child wasn't running");

    let stdout_log: Result<_, LuaError> = stdout_thread.await.expect("stdout thread panicked");
    let stderr_log: Result<_, LuaError> = stderr_thread.await.expect("stderr thread panicked");

    Ok::<_, LuaError>((status, stdout_log?, stderr_log?))
}

pub async fn exit_and_yield_forever(lua: &Lua, exit_code: Option<u8>) -> LuaResult<()> {
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
    time::sleep(Duration::MAX).await;
    Ok(())
}
