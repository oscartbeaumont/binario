use std::collections::{BTreeMap, HashMap};

use tokio::io::AsyncRead;

use crate::{Decode, ReadFixedSizeBuf, ReadLenPrefixedBuf, ReadMap};

impl Decode for u8 {
    type Reader<S: AsyncRead> = ReadFixedSizeBuf<1, Self>;

    fn decode<S: AsyncRead>() -> Self::Reader<S> {
        ReadFixedSizeBuf::new(|v| v[0])
    }
}

// TODO: All number types -> bigger types will need to be encoded over multiple bytes

impl Decode for bool {
    type Reader<S: AsyncRead> = ReadFixedSizeBuf<1, Self>;

    fn decode<S: AsyncRead>() -> Self::Reader<S> {
        ReadFixedSizeBuf::new(|v| v[0] != 0)
    }
}

impl Decode for String {
    type Reader<S: AsyncRead> = ReadLenPrefixedBuf<Self>;

    fn decode<S: AsyncRead>() -> Self::Reader<S> {
        ReadLenPrefixedBuf::new(|v| String::from_utf8(v).unwrap()) // TODO: Error handling
    }
}

impl<T: Decode> Decode for Vec<T> {
    type Reader<S: AsyncRead> = ReadMap<Self>;

    fn decode<S: AsyncRead>() -> Self::Reader<S> {
        ReadMap::new()
    }
}

impl<K: Decode, V: Decode> Decode for HashMap<K, V> {
    type Reader<S: AsyncRead> = ReadMap<Self>;

    fn decode<S: AsyncRead>() -> Self::Reader<S> {
        ReadMap::new()
    }
}

impl<K: Decode, V: Decode> Decode for BTreeMap<K, V> {
    type Reader<S: AsyncRead> = ReadMap<Self>;

    fn decode<S: AsyncRead>() -> Self::Reader<S> {
        ReadMap::new()
    }
}

// TODO: Cow<'b, str>, Vec<T>, &'b T, Arc<T>, [T], [T; N]
