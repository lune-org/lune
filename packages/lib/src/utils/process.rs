// https://stackoverflow.com/questions/71141122/-

use std::{
    io,
    io::Write,
    process::{Child, ExitStatus},
    sync::Weak,
    time::Duration,
};

use mlua::prelude::*;
use tokio::{sync::mpsc::Sender, time};

use crate::LuneMessage;

pub struct TeeWriter<'a, W0: Write, W1: Write> {
    w0: &'a mut W0,
    w1: &'a mut W1,
}

impl<'a, W0: Write, W1: Write> TeeWriter<'a, W0, W1> {
    pub fn new(w0: &'a mut W0, w1: &'a mut W1) -> Self {
        Self { w0, w1 }
    }
}

impl<'a, W0: Write, W1: Write> Write for TeeWriter<'a, W0, W1> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // We have to use write_all() otherwise what
        // happens if different amounts are written?
        self.w0.write_all(buf)?;
        self.w1.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w0.flush()?;
        self.w1.flush()?;
        Ok(())
    }
}

pub fn pipe_and_inherit_child_process_stdio(
    mut child: Child,
) -> LuaResult<(ExitStatus, Vec<u8>, Vec<u8>)> {
    // https://stackoverflow.com/questions/71141122/-
    let mut child_stdout = child.stdout.take().unwrap();
    let mut child_stderr = child.stderr.take().unwrap();
    std::thread::scope(|s| {
        let stdout_thread = s.spawn(|| {
            let stdout = io::stdout();
            let mut log = Vec::new();
            let mut stdout = stdout.lock();
            let mut tee = TeeWriter::new(&mut stdout, &mut log);

            io::copy(&mut child_stdout, &mut tee).map_err(LuaError::external)?;

            Ok(log)
        });

        let stderr_thread = s.spawn(|| {
            let stderr = io::stderr();
            let mut log = Vec::new();
            let mut stderr = stderr.lock();
            let mut tee = TeeWriter::new(&mut stderr, &mut log);

            io::copy(&mut child_stderr, &mut tee).map_err(LuaError::external)?;

            Ok(log)
        });

        let status = child.wait().expect("child wasn't running");

        let stdout_log: Result<_, LuaError> = stdout_thread.join().expect("stdout thread panicked");
        let stderr_log: Result<_, LuaError> = stderr_thread.join().expect("stderr thread panicked");

        Ok::<_, LuaError>((status, stdout_log?, stderr_log?))
    })
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
