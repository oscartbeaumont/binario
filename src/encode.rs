use std::{
    future::poll_fn,
    io,
    pin::{pin, Pin},
    task::{Context, Poll},
};

use pin_project::pin_project;
use tokio::io::AsyncWrite;

/// TODO
pub trait Encode {
    /// Writer is the type that will be used to encode the data.
    /// This should hold a cursor and buffer to keep track of the write progress.
    type Writer<'a>: Writer + 'a
    where
        Self: 'a;

    /// Length will determine how many bytes are needed to encode the data.
    /// This is primarily designed to make map's work however it may be useful to preallocate in-memory buffers.
    fn byte_len(&self) -> usize;

    /// Create a new writer that will be used to encode the current item.
    fn encode<'a>(&'a self) -> Self::Writer<'a>;
}

/// TODO
pub trait Writer {
    /// TODO
    fn poll_writer<S: AsyncWrite>(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        s: Pin<&mut S>,
    ) -> Poll<io::Result<()>>;
}

/// TODO
#[doc(hidden)]
#[pin_project(project = WriterOrDoneProj)]
pub enum WriterOrDone<'a, T>
where
    T: Encode + 'a,
{
    Writer(#[pin] T::Writer<'a>),
    Done,
}

impl<'a, T> WriterOrDone<'a, T>
where
    T: Encode + 'a,
{
    #[inline]
    #[doc(hidden)]
    pub fn unsafe_poll<S: AsyncWrite>(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        s: Pin<&mut S>,
    ) -> Option<Poll<io::Result<()>>> {
        let this = self.as_mut().project();
        match this {
            WriterOrDoneProj::Writer(fut) => match fut.poll_writer(cx, s) {
                Poll::Ready(result) => {
                    self.set(Self::Done);
                    match result {
                        Ok(_) => None,
                        Err(e) => Some(Poll::Ready(Err(e))),
                    }
                }
                Poll::Pending => Some(Poll::Pending),
            },
            WriterOrDoneProj::Done => None,
        }
    }
}

/// TODO
// TODO: Take in 'S' as '&mut S' or at worst have it return the stream?
pub async fn encode<T: Encode, S: AsyncWrite>(t: &T, s: S) -> io::Result<()> {
    let mut s = pin!(s);

    poll_fn(|cx| {
        let writer = t.encode();
        let writer = pin!(writer);
        writer.poll_writer(cx, s.as_mut())
    })
    .await
}
