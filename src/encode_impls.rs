use std::{borrow::Cow, sync::Arc};

use tokio::io::AsyncWrite;

use crate::{Encode, WriteBuf, WriteBuf2, WriteFixedBuf};

impl Encode for u8 {
    type Writer<'a, S: AsyncWrite> = WriteFixedBuf<1>
    where
        Self: 'a;

    fn encode<S: AsyncWrite>(&self) -> Self::Writer<'_, S> where {
        WriteFixedBuf::new([*self])
    }
}

// TODO: All number types -> bigger types will need to be encoded over multiple bytes

impl Encode for bool {
    type Writer<'a, S: AsyncWrite> = WriteFixedBuf<1>
    where
        Self: 'a;

    fn encode<S: AsyncWrite>(&self) -> Self::Writer<'_, S> where {
        WriteFixedBuf::new([*self as u8])
    }
}

impl Encode for &'static str {
    type Writer<'a, S: AsyncWrite> = WriteBuf<'a, u8>
    where
        Self: 'a;

    fn encode<S: AsyncWrite>(&self) -> Self::Writer<'_, S> where {
        WriteBuf::new(self.len(), self.as_bytes())
    }
}

impl Encode for String {
    type Writer<'a, S: AsyncWrite> = WriteBuf2<'a>
    where
        Self: 'a;

    fn encode<S: AsyncWrite>(&self) -> Self::Writer<'_, S> where {
        WriteBuf2::new(self.len(), self.as_bytes())
    }
}

impl<'b> Encode for Cow<'b, str> {
    type Writer<'a, S: AsyncWrite> = WriteBuf<'a, u8>
    where
        Self: 'a;

    fn encode<S: AsyncWrite>(&self) -> Self::Writer<'_, S> where {
        WriteBuf::new(self.len(), self.as_bytes())
    }
}

impl<T: Encode> Encode for Vec<T> {
    type Writer<'a, S: AsyncWrite> = WriteBuf<'a, T>
    where
        Self: 'a;

    fn encode<S: AsyncWrite>(&self) -> Self::Writer<'_, S> where {
        WriteBuf::new(self.len(), self)
    }
}

impl<'b, T: Encode + 'b> Encode for &'b T {
    type Writer<'a, S: AsyncWrite> = T::Writer<'a, S>
    where
        Self: 'a;

    fn encode<S: AsyncWrite>(&self) -> Self::Writer<'_, S> where {
        T::encode(self)
    }
}

impl<T: Encode> Encode for Arc<T> {
    type Writer<'a, S: AsyncWrite> = T::Writer<'a, S>
    where
        Self: 'a;

    fn encode<S: AsyncWrite>(&self) -> Self::Writer<'_, S> where {
        T::encode(self)
    }
}

// TODO: Rc, RefCell, etc

impl<T: Encode> Encode for [T] {
    type Writer<'a, S: AsyncWrite> = WriteBuf<'a, T>
    where
        Self: 'a;

    fn encode<S: AsyncWrite>(&self) -> Self::Writer<'_, S> where {
        WriteBuf::new(self.len(), self)
    }
}

impl<const N: usize, T: Encode> Encode for [T; N] {
    type Writer<'a, S: AsyncWrite> = WriteBuf<'a, T>
    where
        Self: 'a;

    fn encode<S: AsyncWrite>(&self) -> Self::Writer<'_, S> where {
        WriteBuf::new(N, self)
    }
}
