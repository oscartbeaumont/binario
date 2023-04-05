use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::io::{AsyncRead, ReadBuf as TokioReadBuf};

macro_rules! ready {
    ($e:expr $(,)?) => {
        match $e {
            std::task::Poll::Ready(t) => t,
            std::task::Poll::Pending => return std::task::Poll::Pending,
        }
    };
}

use crate::Reader;

fn eof() -> io::Error {
    io::Error::new(io::ErrorKind::UnexpectedEof, "early eof")
}

pub struct ReadFixedSizeBuf<const N: usize, T> {
    buf: [u8; N],
    map: fn([u8; N]) -> T,
}

impl<const N: usize, T> ReadFixedSizeBuf<N, T> {
    pub fn new(map: fn([u8; N]) -> T) -> Self {
        Self { buf: [0; N], map }
    }
}

impl<const N: usize, S: AsyncRead, T> Reader<S> for ReadFixedSizeBuf<N, T> {
    type T = T;

    fn poll_reader(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut s: Pin<&mut S>,
    ) -> Poll<std::io::Result<Self::T>> {
        let mut buf = TokioReadBuf::new(&mut self.buf);
        loop {
            let rem = buf.remaining();
            if rem != 0 {
                ready!(s.as_mut().poll_read(cx, &mut buf))?;
                if buf.remaining() == rem {
                    return Err(eof()).into();
                }
            } else {
                return Poll::Ready(Ok((self.map)(self.buf)));
            }
        }
    }
}

pub struct ReadLenPrefixedBuf<T> {
    state: ReadBufState,
    map: fn(Vec<u8>) -> T,
}

enum ReadBufState {
    None,
    AllocatedBuf(Vec<u8>),
    Done,
}

impl<T> ReadLenPrefixedBuf<T> {
    pub fn new(map: fn(Vec<u8>) -> T) -> Self {
        Self {
            state: ReadBufState::None,
            map,
        }
    }
}

impl<S: AsyncRead, T> Reader<S> for ReadLenPrefixedBuf<T> {
    type T = T;

    fn poll_reader(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut s: Pin<&mut S>,
    ) -> Poll<std::io::Result<Self::T>> {
        let buf = loop {
            match &mut self.state {
                ReadBufState::None => {
                    let mut buf = [0u8; 1];
                    let mut buf = TokioReadBuf::new(&mut buf);
                    ready!(s.as_mut().poll_read(cx, &mut buf))?;

                    if buf.remaining() == 0 {
                        let len = buf.filled()[0] as usize; // TODO: More than a usize for length
                        let buf = vec![0; len]; // TODO: Can we avoid zeroing out the array cause it might help with performance???

                        self.state = ReadBufState::AllocatedBuf(buf);
                        continue;
                    } else {
                        // A buffer will only ever return nothing if it's the end of the file.
                        return Err(eof()).into();
                    }
                }
                ReadBufState::AllocatedBuf(b) => break b,
                ReadBufState::Done => panic!("Future polled after completion"),
            }
        };

        let mut buf = TokioReadBuf::new(buf);
        loop {
            let rem = buf.remaining();
            if rem != 0 {
                ready!(s.as_mut().poll_read(cx, &mut buf))?;
                if buf.remaining() == rem {
                    return Err(eof()).into();
                }
            } else {
                let buf = match std::mem::replace(&mut self.state, ReadBufState::Done) {
                    ReadBufState::AllocatedBuf(b) => b,
                    _ => unreachable!(),
                };

                return Poll::Ready(Ok((self.map)(buf)));
            }
        }
    }
}
