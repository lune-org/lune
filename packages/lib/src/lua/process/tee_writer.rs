use std::{
    io::Write,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;
use tokio::io::{self, AsyncWrite};

#[pin_project]
pub struct AsyncTeeWriter<'a, W>
where
    W: AsyncWrite + Unpin,
{
    #[pin]
    writer: &'a mut W,
    buffer: Vec<u8>,
}

impl<'a, W> AsyncTeeWriter<'a, W>
where
    W: AsyncWrite + Unpin,
{
    pub fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            buffer: Vec::new(),
        }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buffer
    }
}

impl<'a, W> AsyncWrite for AsyncTeeWriter<'a, W>
where
    W: AsyncWrite + Unpin,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let mut this = self.project();
        match this.writer.as_mut().poll_write(cx, buf) {
            Poll::Ready(res) => {
                this.buffer
                    .write_all(buf)
                    .expect("Failed to write to internal tee buffer");
                Poll::Ready(res)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().writer.as_mut().poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().writer.as_mut().poll_flush(cx)
    }
}
