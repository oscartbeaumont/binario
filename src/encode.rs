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
    type Writer<'a, S: AsyncWrite + 'a>: Writer<S> + 'a
    where
        Self: 'a;

    fn encode<'a, S: AsyncWrite + 'a>(&'a self) -> Self::Writer<'a, S>;
}

/// TODO
pub trait Writer<S> {
    fn poll_writer(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        s: Pin<&mut S>,
    ) -> Poll<io::Result<()>>;
}

/// TODO
#[doc(hidden)]
#[pin_project(project = WriterOrDoneProj)]
pub enum WriterOrDone<'a, T, S>
where
    T: Encode + 'a,
    S: AsyncWrite + 'a,
{
    Writer(#[pin] T::Writer<'a, S>),
    Done,
}

impl<'a, T, S> WriterOrDone<'a, T, S>
where
    T: Encode + 'a,
    S: AsyncWrite,
{
    #[inline]
    #[doc(hidden)]
    pub fn unsafe_poll(
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
