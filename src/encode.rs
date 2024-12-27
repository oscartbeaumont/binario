use std::{
    future::Future,
    io,
    pin::{pin, Pin},
};

use tokio::io::AsyncWrite;

/// TODO
pub trait Encode {
    /// Poll the encoding of the current item.
    fn encode<S: AsyncWrite>(&self, s: Pin<&mut S>) -> impl Future<Output = io::Result<()>>;

    /// Length will determine how many bytes are needed to encode the data.
    /// This primarily exists to make map's work however it may be useful to preallocate in-memory buffers.
    fn byte_len(&self) -> usize;
}

/// TODO
pub async fn encode<T: Encode, S: AsyncWrite>(t: &T, s: S) -> io::Result<()> {
    let mut s = pin!(s);
    t.encode(s.as_mut()).await
}
