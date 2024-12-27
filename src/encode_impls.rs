use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    io,
    mem::size_of,
    pin::Pin,
    sync::Arc,
};

use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::Encode;

const LEN_SIZE: usize = size_of::<u64>();

impl Encode for u8 {
    async fn encode<S: AsyncWrite>(&self, mut s: Pin<&mut S>) -> io::Result<()> {
        s.write_u8(*self).await
    }

    fn byte_len(&self) -> usize {
        1
    }
}

// TODO: All number types -> bigger types will need to be encoded over multiple bytes

impl Encode for bool {
    async fn encode<S: AsyncWrite>(&self, mut s: Pin<&mut S>) -> io::Result<()> {
        s.write_u8(*self as u8).await
    }

    fn byte_len(&self) -> usize {
        1
    }
}

impl Encode for &'static str {
    async fn encode<S: AsyncWrite>(&self, mut s: Pin<&mut S>) -> io::Result<()> {
        let len: u64 = self.len().try_into().unwrap();
        s.write_all(&len.to_le_bytes()).await?;
        s.write_all(self.as_bytes()).await
    }

    fn byte_len(&self) -> usize {
        LEN_SIZE + self.len()
    }
}

impl Encode for String {
    async fn encode<S: AsyncWrite>(&self, mut s: Pin<&mut S>) -> io::Result<()> {
        let len: u64 = self.len().try_into().unwrap();
        s.write_all(&len.to_le_bytes()).await?;
        s.write_all(self.as_bytes()).await
    }

    fn byte_len(&self) -> usize {
        LEN_SIZE + self.len()
    }
}

impl<'b> Encode for Cow<'b, str> {
    async fn encode<S: AsyncWrite>(&self, mut s: Pin<&mut S>) -> io::Result<()> {
        let len: u64 = self.len().try_into().unwrap();
        s.write_all(&len.to_le_bytes()).await?;
        s.write_all(self.as_bytes()).await
    }

    fn byte_len(&self) -> usize {
        LEN_SIZE + self.len()
    }
}

impl<T: Encode> Encode for Vec<T> {
    async fn encode<S: AsyncWrite>(&self, mut s: Pin<&mut S>) -> io::Result<()> {
        let len: u64 = self.len().try_into().unwrap();
        s.write_all(&len.to_le_bytes()).await?;
        for item in self {
            item.encode(s.as_mut()).await?;
        }
        Ok(())
    }

    fn byte_len(&self) -> usize {
        LEN_SIZE + self.iter().map(|i| i.byte_len()).sum::<usize>()
    }
}

impl<'b, T: Encode + 'b> Encode for &'b T {
    async fn encode<S: AsyncWrite>(&self, mut s: Pin<&mut S>) -> io::Result<()> {
        T::encode(self, s.as_mut()).await
    }

    fn byte_len(&self) -> usize {
        T::byte_len(self)
    }
}

impl<T: Encode> Encode for Arc<T> {
    async fn encode<S: AsyncWrite>(&self, mut s: Pin<&mut S>) -> io::Result<()> {
        T::encode(self, s.as_mut()).await
    }

    fn byte_len(&self) -> usize {
        T::byte_len(self)
    }
}

// TODO: Rc, RefCell, etc

impl<T: Encode> Encode for [T] {
    async fn encode<S: AsyncWrite>(&self, mut s: Pin<&mut S>) -> io::Result<()> {
        let len: u64 = self.len().try_into().unwrap();
        s.write_all(&len.to_le_bytes()).await?;
        for item in self {
            item.encode(s.as_mut()).await?;
        }
        Ok(())
    }

    fn byte_len(&self) -> usize {
        LEN_SIZE + self.iter().map(|i| i.byte_len()).sum::<usize>()
    }
}

impl<const N: usize, T: Encode> Encode for [T; N] {
    async fn encode<S: AsyncWrite>(&self, mut s: Pin<&mut S>) -> io::Result<()> {
        let len: u64 = self.len().try_into().unwrap();
        s.write_all(&len.to_le_bytes()).await?;
        for item in self {
            item.encode(s.as_mut()).await?;
        }
        Ok(())
    }

    fn byte_len(&self) -> usize {
        LEN_SIZE + self.iter().map(|i| i.byte_len()).sum::<usize>()
    }
}

impl<K: Encode, V: Encode> Encode for HashMap<K, V> {
    async fn encode<S: AsyncWrite>(&self, mut s: Pin<&mut S>) -> io::Result<()> {
        let len: u64 = self.len().try_into().unwrap();
        s.write_all(&len.to_le_bytes()).await?;
        for (k, v) in self {
            // TODO: Are the key and value lengths needed?
            let k_len: u64 = k.byte_len().try_into().unwrap();
            let v_len: u64 = v.byte_len().try_into().unwrap();

            s.write_all(&k_len.to_le_bytes()).await?;
            k.encode(s.as_mut()).await?;

            s.write_all(&v_len.to_le_bytes()).await?;
            v.encode(s.as_mut()).await?;
        }
        Ok(())
    }

    fn byte_len(&self) -> usize {
        // Length of map, key length, value body length, value length, key body length
        LEN_SIZE
            + self
                .iter()
                .map(|(k, v)| LEN_SIZE + k.byte_len() + LEN_SIZE + v.byte_len())
                .sum::<usize>()
    }
}

impl<K: Encode, V: Encode> Encode for BTreeMap<K, V> {
    async fn encode<S: AsyncWrite>(&self, mut s: Pin<&mut S>) -> io::Result<()> {
        let len: u64 = self.len().try_into().unwrap();
        s.write_all(&len.to_le_bytes()).await?;
        for (k, v) in self {
            // TODO: Are the key and value lengths needed?
            let k_len: u64 = k.byte_len().try_into().unwrap();
            let v_len: u64 = v.byte_len().try_into().unwrap();

            s.write_all(&k_len.to_le_bytes()).await?;
            k.encode(s.as_mut()).await?;

            s.write_all(&v_len.to_le_bytes()).await?;
            v.encode(s.as_mut()).await?;
        }
        Ok(())
    }

    fn byte_len(&self) -> usize {
        // Length of map, key length, value body length, value length, key body length
        LEN_SIZE
            + self
                .iter()
                .map(|(k, v)| LEN_SIZE + k.byte_len() + LEN_SIZE + v.byte_len())
                .sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use crate::encode_impls::LEN_SIZE;

    #[test]
    fn test_len() {
        // The code assumes this in many places. Try and avoid that then maybe remove this test!
        assert_eq!(LEN_SIZE, 8);
    }
}
