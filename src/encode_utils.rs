use std::{
    io,
    pin::{pin, Pin},
    task::{Context, Poll},
};

use tokio::io::AsyncWrite;

use crate::{Encode, Writer};

/// TODO
pub struct WriteFixedBuf<const N: usize> {
    buf: [u8; N],
    cursor: usize,
}

impl<const N: usize> WriteFixedBuf<N> {
    pub fn new(buf: [u8; N]) -> Self {
        Self { buf, cursor: 0 }
    }
}

impl<const N: usize, S: AsyncWrite> Writer<S> for WriteFixedBuf<N> {
    fn poll_writer(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut s: Pin<&mut S>,
    ) -> Poll<io::Result<()>> {
        loop {
            match s
                .as_mut()
                .poll_write(cx, &self.buf[self.cursor..self.buf.len()])
            {
                Poll::Ready(Ok(n)) => {
                    self.cursor += n;

                    if n == 0 {
                        return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
                    } else if self.cursor == self.buf.len() {
                        return Poll::Ready(Ok(()));
                    }
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// TODO
pub struct WriteBuf<'a, T: Encode> {
    len_buf: [u8; 8],
    len_cursor: usize,
    buf: &'a [T],
    cursor: usize,
}

impl<'a, T: Encode> WriteBuf<'a, T> {
    pub fn new(len: usize, buf: &'a [T]) -> Self {
        let len: u64 = len.try_into().unwrap();
        Self {
            len_buf: len.to_le_bytes(), // x86_64 uses little endian so we gonna stick with it
            len_cursor: 0,
            buf,
            cursor: 0,
        }
    }
}

impl<'a, T: Encode, S: AsyncWrite> Writer<S> for WriteBuf<'a, T> {
    fn poll_writer(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut s: Pin<&mut S>,
    ) -> Poll<io::Result<()>> {
        loop {
            if self.len_cursor == 8 {
                break;
            }

            match s.as_mut().poll_write(cx, &self.len_buf) {
                Poll::Ready(Ok(n)) => {
                    self.len_cursor += n;
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        loop {
            // TODO: Optimised encode method that works on `Vec<T>` where `T: Encode` so that we don't need `WriteBuf2`
            let w: T::Writer<'_, S> = self.buf[self.cursor].encode();
            let w = pin!(w);

            match w.poll_writer(cx, s.as_mut()) {
                Poll::Ready(Ok(_)) => {
                    self.cursor += 1;

                    if self.cursor == self.buf.len() {
                        return Poll::Ready(Ok(()));
                    }
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// TODO
pub struct WriteBuf2<'a> {
    len_buf: [u8; 8],
    len_cursor: usize,
    buf: &'a [u8],
    cursor: usize,
}

impl<'a> WriteBuf2<'a> {
    pub fn new(len: usize, buf: &'a [u8]) -> Self {
        let len: u64 = len.try_into().unwrap();
        Self {
            len_buf: len.to_le_bytes(), // x86_64 uses little endian so we gonna stick with it
            len_cursor: 0,
            buf,
            cursor: 0,
        }
    }
}

impl<'a, S: AsyncWrite> Writer<S> for WriteBuf2<'a> {
    fn poll_writer(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut s: Pin<&mut S>,
    ) -> Poll<io::Result<()>> {
        loop {
            if self.len_cursor == 8 {
                break;
            }

            match s.as_mut().poll_write(cx, &self.len_buf) {
                Poll::Ready(Ok(n)) => {
                    self.len_cursor += n;
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        loop {
            match s
                .as_mut()
                .poll_write(cx, &self.buf[self.cursor..self.buf.len()])
            {
                Poll::Ready(Ok(n)) => {
                    self.cursor += n;

                    if n == 0 {
                        return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
                    } else if self.cursor == self.buf.len() {
                        return Poll::Ready(Ok(()));
                    }
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}
