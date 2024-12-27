use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    io,
    pin::Pin,
};

use tokio::io::{AsyncRead, AsyncReadExt};

use crate::Decode;

impl Decode for u8 {
    async fn decode<S: AsyncRead>(mut s: Pin<&mut S>) -> io::Result<Self> {
        s.read_u8().await
    }
}

// TODO: All number types -> bigger types will need to be encoded over multiple bytes

impl Decode for bool {
    async fn decode<S: AsyncRead>(mut s: Pin<&mut S>) -> io::Result<Self> {
        s.read_u8().await.map(|b| b != 0)
    }
}

impl Decode for String {
    async fn decode<S: AsyncRead>(mut s: Pin<&mut S>) -> io::Result<Self> {
        let len = s.read_u64_le().await?;
        let mut buf = vec![0; len as usize];
        s.read_exact(&mut buf).await?;
        String::from_utf8(buf)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid utf-8"))
    }
}

impl<T: Decode> Decode for Vec<T> {
    async fn decode<S: AsyncRead>(mut s: Pin<&mut S>) -> io::Result<Self> {
        let len = s.read_u64_le().await?;
        let mut vec = Vec::with_capacity(len as usize);
        for _ in 0..len {
            vec.push(T::decode(s.as_mut()).await?);
        }
        Ok(vec)
    }
}

impl<K: Decode + Hash + Eq, V: Decode> Decode for HashMap<K, V> {
    async fn decode<S: AsyncRead>(mut s: Pin<&mut S>) -> io::Result<Self> {
        let len = s.read_u64_le().await?;
        let mut map = HashMap::with_capacity(len as usize);
        for _ in 0..len {
            let key = K::decode(s.as_mut()).await?;
            let value = V::decode(s.as_mut()).await?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

impl<K: Decode + Ord, V: Decode> Decode for BTreeMap<K, V> {
    async fn decode<S: AsyncRead>(mut s: Pin<&mut S>) -> io::Result<Self> {
        let len = s.read_u64_le().await?;
        let mut map = BTreeMap::new();
        for _ in 0..len {
            let key = K::decode(s.as_mut()).await?;
            let value = V::decode(s.as_mut()).await?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

// TODO: Cow<'b, str>, Vec<T>, &'b T, Arc<T>, [T], [T; N]
