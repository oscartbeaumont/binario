use std::{
    future::poll_fn,
    io,
    pin::{pin, Pin},
    task::{Context, Poll},
};

use tokio::io::AsyncWrite;

/// TODO
pub trait Encode {
    type Writer<'a, S: AsyncWrite>: Writer<S> + 'a
    where
        Self: 'a;

    fn encode<S: AsyncWrite>(&self) -> Self::Writer<'_, S>;
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
