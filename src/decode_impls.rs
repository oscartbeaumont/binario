use tokio::io::AsyncRead;

use crate::{Decode, ReadFixedSizeBuf, ReadLenPrefixedBuf};

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
        ReadLenPrefixedBuf::new(|v| {
            // println!("v: {:?}", v.len()); // TODO
            String::from_utf8(v).unwrap()
        }) // TODO: Error handling
    }
}

// impl<T: Decode> Decode for Vec<T> {
//     type Reader<S: AsyncRead> = ReadLenPrefixedBuf<Self>;

//     fn decode<S: AsyncRead>() -> Self::Reader<S> {
//         ReadLenPrefixedBuf::new(|v| {
//             // println!("v: {:?}", v.len()); // TODO
//             String::from_utf8(v).unwrap()
//         }) // TODO: Error handling
//     }
// }

// TODO: Cow<'b, str>, Vec<T>, &'b T, Arc<T>, [T], [T; N]
