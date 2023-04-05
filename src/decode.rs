use std::{
    future::poll_fn,
    io, mem,
    pin::{pin, Pin},
    task::{Context, Poll},
};

use pin_project::pin_project;
use tokio::io::AsyncRead;

/// TODO
pub trait Decode {
    type Reader<S: AsyncRead>: Reader<S, T = Self>;

    fn decode<S: AsyncRead>() -> Self::Reader<S>;
}

/// TODO
pub trait Reader<S> {
    type T;

    fn poll_reader(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        s: Pin<&mut S>,
    ) -> Poll<io::Result<Self::T>>;
}

/// TODO
#[doc(hidden)]
#[pin_project(project = ValueOrWriterProj)]
pub enum ValueOrReader<T, S>
where
    T: Decode,
    S: AsyncRead,
{
    Reader(#[pin] T::Reader<S>),
    Value(T),
    Done,
}

impl<T, S> ValueOrReader<T, S>
where
    T: Decode,
    S: AsyncRead,
{
    #[inline]
    #[doc(hidden)]
    pub fn unsafe_poll<R>(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        s: Pin<&mut S>,
    ) -> Option<Poll<io::Result<R>>> {
        let this = self.as_mut().project();
        match this {
            ValueOrWriterProj::Reader(fut) => match fut.poll_reader(cx, s) {
                Poll::Ready(Ok(v)) => {
                    self.set(ValueOrReader::Value(v));
                    None
                }
                Poll::Ready(Err(err)) => {
                    self.set(ValueOrReader::Done);
                    Some(Poll::Ready(Err(err)))
                }
                Poll::Pending => Some(Poll::Pending),
            },
            ValueOrWriterProj::Value(_) => None,
            ValueOrWriterProj::Done => {
                panic!("ValueOrWriter future polled after completion");
            }
        }
    }

    pub fn unsafe_take(&mut self) -> T {
        match mem::replace(self, ValueOrReader::Done) {
            Self::Value(v) => v,
            _ => unreachable!(),
        }
    }
}

/// TODO
// TODO: Take in 'S' as '&mut S' or at worst have it return the stream?
pub async fn decode<T: Decode, S: AsyncRead>(s: S) -> io::Result<T> {
    let mut s = pin!(s);

    poll_fn(|cx| {
        let writer = T::decode();
        let writer = pin!(writer);
        writer.poll_reader(cx, s.as_mut())
    })
    .await
}
