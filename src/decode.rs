use std::{
    future::poll_fn,
    io,
    pin::{pin, Pin},
    task::{Context, Poll},
};

use tokio::io::AsyncRead;

/// TODO
pub trait Decode {
    type Writer<S: AsyncRead>: Reader<S, T = Self>;

    fn decode<S: AsyncRead>() -> Self::Writer<S>;
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
