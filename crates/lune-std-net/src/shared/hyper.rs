use std::{
    future::Future,
    io,
    pin::Pin,
    slice,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use async_io::Timer;
use futures_lite::{prelude::*, ready};
use hyper::rt::{self, Executor, ReadBuf, ReadBufCursor};
use mlua::prelude::*;
use mlua_luau_scheduler::LuaSpawnExt;

// Hyper executor that spawns futures onto our Lua scheduler

#[derive(Debug, Clone)]
pub struct HyperExecutor {
    lua: Lua,
}

#[allow(dead_code)]
impl HyperExecutor {
    pub fn attach(lua: &Lua) -> mlua::AppDataRef<'_, Self> {
        lua.set_app_data(Self { lua: lua.clone() });
        lua.app_data_ref::<Self>().unwrap()
    }

    pub fn execute<Fut>(lua: Lua, fut: Fut)
    where
        Fut: Future + Send + 'static,
        Fut::Output: Send + 'static,
    {
        let exec = lua
            .app_data_ref::<Self>()
            .unwrap_or_else(|| Self::attach(&lua));

        exec.execute(fut);
    }
}

impl<Fut: Future + Send + 'static> rt::Executor<Fut> for HyperExecutor
where
    Fut::Output: Send + 'static,
{
    fn execute(&self, fut: Fut) {
        self.lua.spawn(fut).detach();
    }
}

// Hyper timer & sleep future wrapper for async-io

#[derive(Debug)]
pub struct HyperTimer;

impl rt::Timer for HyperTimer {
    fn sleep(&self, duration: Duration) -> Pin<Box<dyn rt::Sleep>> {
        Box::pin(HyperSleep::from(Timer::after(duration)))
    }

    fn sleep_until(&self, at: Instant) -> Pin<Box<dyn rt::Sleep>> {
        Box::pin(HyperSleep::from(Timer::at(at)))
    }

    fn reset(&self, sleep: &mut Pin<Box<dyn rt::Sleep>>, new_deadline: Instant) {
        if let Some(mut sleep) = sleep.as_mut().downcast_mut_pin::<HyperSleep>() {
            sleep.inner.set_at(new_deadline);
        } else {
            *sleep = Box::pin(HyperSleep::from(Timer::at(new_deadline)));
        }
    }
}

#[derive(Debug)]
pub struct HyperSleep {
    inner: Timer,
}

impl From<Timer> for HyperSleep {
    fn from(inner: Timer) -> Self {
        Self { inner }
    }
}

impl Future for HyperSleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        match Pin::new(&mut self.inner).poll(cx) {
            Poll::Ready(_) => Poll::Ready(()),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl rt::Sleep for HyperSleep {}

// Hyper I/O wrapper for bidirectional compatibility
// between hyper & futures-lite async read/write traits

pin_project_lite::pin_project! {
    #[derive(Debug)]
    pub struct HyperIo<T> {
        #[pin]
        inner: T
    }
}

impl<T> From<T> for HyperIo<T> {
    fn from(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> HyperIo<T> {
    pub fn pin_mut(self: Pin<&mut Self>) -> Pin<&mut T> {
        self.project().inner
    }
}

// Compat for futures-lite -> hyper runtime

impl<T: AsyncRead> rt::Read for HyperIo<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: ReadBufCursor<'_>,
    ) -> Poll<io::Result<()>> {
        // Fill the read buffer with initialized data
        let read_slice = unsafe {
            let buffer = buf.as_mut();
            buffer.as_mut_ptr().write_bytes(0, buffer.len());
            slice::from_raw_parts_mut(buffer.as_mut_ptr().cast::<u8>(), buffer.len())
        };

        // Read bytes from the underlying source
        let n = match self.pin_mut().poll_read(cx, read_slice) {
            Poll::Ready(Ok(n)) => n,
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Pending => return Poll::Pending,
        };

        unsafe {
            buf.advance(n);
        }

        Poll::Ready(Ok(()))
    }
}

impl<T: AsyncWrite> rt::Write for HyperIo<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.pin_mut().poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.pin_mut().poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.pin_mut().poll_close(cx)
    }
}

// Compat for hyper runtime -> futures-lite

impl<T: rt::Read> AsyncRead for HyperIo<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let mut buf = ReadBuf::new(buf);
        ready!(self.pin_mut().poll_read(cx, buf.unfilled()))?;
        Poll::Ready(Ok(buf.filled().len()))
    }
}

impl<T: rt::Write> AsyncWrite for HyperIo<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        self.pin_mut().poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        self.pin_mut().poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.pin_mut().poll_shutdown(cx)
    }
}
