use std::{
    future::Future,
    io,
    pin::{pin, Pin},
};

use tokio::io::AsyncRead;

/// TODO
pub trait Decode: Sized {
    fn decode<S: AsyncRead>(reader: Pin<&mut S>) -> impl Future<Output = io::Result<Self>>;
}

/// TODO
pub async fn decode<T: Decode, S: AsyncRead>(s: S) -> io::Result<T> {
    let s = pin!(s);
    T::decode(s).await
}
