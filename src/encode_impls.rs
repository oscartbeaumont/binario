use std::{
    borrow::Cow,
    collections::{btree_map, hash_map, BTreeMap, HashMap},
    mem::size_of,
    sync::Arc,
};

use crate::{Encode, WriteBuf, WriteBuf2, WriteFixedBuf, WriteMap};

const LEN_SIZE: usize = size_of::<u64>();

impl Encode for u8 {
    type Writer<'a> = WriteFixedBuf<1>
    where
        Self: 'a;

    fn byte_len(&self) -> usize {
        1
    }

    fn encode<'a>(&'a self) -> Self::Writer<'a> {
        WriteFixedBuf::new([*self])
    }
}

// TODO: All number types -> bigger types will need to be encoded over multiple bytes

impl Encode for bool {
    type Writer<'a> = WriteFixedBuf<1>
    where
        Self: 'a;

    fn byte_len(&self) -> usize {
        1
    }

    fn encode<'a>(&'a self) -> Self::Writer<'a> {
        WriteFixedBuf::new([*self as u8])
    }
}

impl Encode for &'static str {
    type Writer<'a> = WriteBuf<'a, u8>
    where
        Self: 'a;

    fn byte_len(&self) -> usize {
        1
    }

    fn encode<'a>(&'a self) -> Self::Writer<'a> {
        WriteBuf::new(self.byte_len(), self.as_bytes())
    }
}

impl Encode for String {
    type Writer<'a> = WriteBuf2<'a>
    where
        Self: 'a;

    fn byte_len(&self) -> usize {
        LEN_SIZE + self.len()
    }

    fn encode<'a>(&'a self) -> Self::Writer<'a> {
        WriteBuf2::new(self.len(), self.as_bytes())
    }
}

impl<'b> Encode for Cow<'b, str> {
    type Writer<'a> = WriteBuf<'a, u8>
    where
        Self: 'a;

    fn byte_len(&self) -> usize {
        // LEN_SIZE
        //     + match self {
        //         Cow::Borrowed(s) => s.len(),
        //         Cow::Owned(s) => s.len(),
        //     }
        todo!();
    }

    fn encode<'a>(&'a self) -> Self::Writer<'a> {
        WriteBuf::new(self.byte_len(), self.as_bytes())
    }
}

impl<T: Encode> Encode for Vec<T> {
    type Writer<'a> = WriteBuf<'a, T>
    where
        Self: 'a;

    fn byte_len(&self) -> usize {
        LEN_SIZE + self.len()
    }

    fn encode<'a>(&'a self) -> Self::Writer<'a> {
        WriteBuf::new(self.len(), self)
    }
}

impl<'b, T: Encode + 'b> Encode for &'b T {
    type Writer<'a> = T::Writer<'a>
    where
        Self: 'a;

    fn byte_len(&self) -> usize {
        T::byte_len(&self)
    }

    fn encode<'a>(&'a self) -> Self::Writer<'a> {
        T::encode(self)
    }
}

impl<T: Encode> Encode for Arc<T> {
    type Writer<'a> = T::Writer<'a>
    where
        Self: 'a;

    fn byte_len(&self) -> usize {
        T::byte_len(&self)
    }

    fn encode<'a>(&'a self) -> Self::Writer<'a> {
        T::encode(self)
    }
}

// TODO: Rc, RefCell, etc

impl<T: Encode> Encode for [T] {
    type Writer<'a> = WriteBuf<'a, T>
    where
        Self: 'a;

    fn byte_len(&self) -> usize {
        LEN_SIZE + self.len()
    }

    fn encode<'a>(&'a self) -> Self::Writer<'a> {
        WriteBuf::new(self.len(), self)
    }
}

impl<const N: usize, T: Encode> Encode for [T; N] {
    type Writer<'a> = WriteBuf<'a, T>
    where
        Self: 'a;

    fn byte_len(&self) -> usize {
        LEN_SIZE + N
    }

    fn encode<'a>(&'a self) -> Self::Writer<'a> {
        WriteBuf::new(N, self)
    }
}

impl<K: Encode, V: Encode> Encode for HashMap<K, V> {
    type Writer<'a> = WriteMap<'a, hash_map::Iter<'a, K, V>, K, V>
    where
        Self: 'a;

    fn byte_len(&self) -> usize {
        // Length of map, key length, value body length, value length, key body length
        LEN_SIZE
            + self
                .iter()
                .map(|(k, v)| LEN_SIZE + k.byte_len() + LEN_SIZE + v.byte_len())
                .sum::<usize>()
    }

    fn encode<'a>(&'a self) -> Self::Writer<'a> {
        WriteMap::new(self.len(), self.iter())
    }
}

impl<K: Encode, V: Encode> Encode for BTreeMap<K, V> {
    type Writer<'a> = WriteMap<'a, btree_map::Iter<'a, K, V>, K, V>
    where
        Self: 'a;

    fn byte_len(&self) -> usize {
        // Length of map, key length, value body length, value length, key body length
        LEN_SIZE
            + self
                .iter()
                .map(|(k, v)| LEN_SIZE + k.byte_len() + LEN_SIZE + v.byte_len())
                .sum::<usize>()
    }

    fn encode<'a>(&'a self) -> Self::Writer<'a> {
        WriteMap::new(self.len(), self.iter())
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
