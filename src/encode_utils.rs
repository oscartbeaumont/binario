use std::{
    io,
    pin::{pin, Pin},
    task::{Context, Poll},
};

use tokio::io::AsyncWrite;

use crate::{Encode, Writer};

/// TODO
pub struct WriteFixedBuf<const N: usize>([u8; N]);

impl<const N: usize> WriteFixedBuf<N> {
    pub fn new(buf: [u8; N]) -> Self {
        Self(buf)
    }
}

impl<const N: usize, S: AsyncWrite> Writer<S> for WriteFixedBuf<N> {
    fn poll_writer(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        s: Pin<&mut S>,
    ) -> Poll<io::Result<()>> {
        s.poll_write(cx, &self.0).map(|result| result.map(|_| ())) // TODO: Deal with number of bytes written which is returned
    }
}

/// TODO
pub struct WriteBuf<'a, T: Encode> {
    len: usize,
    cursor: usize,
    buf: &'a [T],
}

impl<'a, T: Encode> WriteBuf<'a, T> {
    pub fn new(len: usize, buf: &'a [T]) -> Self {
        Self {
            len,
            cursor: 0,
            buf,
        }
    }
}

impl<'a, T: Encode, S: AsyncWrite> Writer<S> for WriteBuf<'a, T> {
    fn poll_writer(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut s: Pin<&mut S>,
    ) -> Poll<io::Result<()>> {
        if self.cursor == 0 {
            // TODO: This is gonna be an integer overflow so fix that
            // TODO: Don't use `usize` for wire format cause it's platform specific
            match s.as_mut().poll_write(cx, &[self.len as u8]) {
                Poll::Ready(Ok(n)) => {
                    if n == 1 {
                        self.cursor += 1;
                    } else {
                        println!("PENDING WRITE");
                        return Poll::Pending; // TODO: Is this correct?
                    }
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        // TODO: Optimised encode method that works on `Vec<T>` where `T: Encode` so that we don't need `WriteBuf2`

        let mut buf_offset = self.cursor - 1;
        // TODO: Remove this loop
        loop {
            let w: T::Writer<'_, S> = self.buf[buf_offset].encode();
            let w = pin!(w);

            match w.poll_writer(cx, s.as_mut()) {
                Poll::Ready(Ok(())) => {
                    self.cursor += 1;
                    buf_offset += 1;

                    if buf_offset == self.len {
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
    len: usize,
    cursor: usize,
    buf: &'a [u8],
}

impl<'a> WriteBuf2<'a> {
    pub fn new(len: usize, buf: &'a [u8]) -> Self {
        Self {
            len,
            cursor: 0,
            buf,
        }
    }
}

impl<'a, S: AsyncWrite> Writer<S> for WriteBuf2<'a> {
    fn poll_writer(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut s: Pin<&mut S>,
    ) -> Poll<io::Result<()>> {
        if self.cursor == 0 {
            // TODO: This is gonna be an integer overflow so fix that
            // TODO: Don't use `usize` for wire format cause it's platform specific
            match s.as_mut().poll_write(cx, &[self.len as u8]) {
                Poll::Ready(Ok(n)) => {
                    if n == 1 {
                        self.cursor += 1;
                    } else {
                        println!("PENDING WRITE");
                        return Poll::Pending; // TODO: Is this correct?
                    }
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        let buf_offset = self.cursor - 1;
        match s.poll_write(cx, &self.buf[buf_offset..self.len]) {
            Poll::Ready(Ok(n)) => {
                self.cursor += n;

                if self.cursor == self.len + 1 {
                    return Poll::Ready(Ok(()));
                }

                return Poll::Pending;
            }
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Pending => return Poll::Pending,
        }
    }
}
