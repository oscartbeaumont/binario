use std::{
    io,
    marker::PhantomData,
    mem,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::io::{AsyncRead, ReadBuf as TokioReadBuf};

macro_rules! ready {
    ($e:expr $(,)?) => {
        match $e {
            std::task::Poll::Ready(t) => t,
            std::task::Poll::Pending => return std::task::Poll::Pending,
        }
    };
}

use crate::{Decode, Reader};

fn eof() -> io::Error {
    io::Error::new(io::ErrorKind::UnexpectedEof, "early eof")
}

pub struct ReadFixedSizeBuf<const N: usize, T> {
    buf: [u8; N],
    map: fn([u8; N]) -> T,
}

impl<const N: usize, T> ReadFixedSizeBuf<N, T> {
    pub fn new(map: fn([u8; N]) -> T) -> Self {
        Self { buf: [0; N], map }
    }
}

impl<const N: usize, S: AsyncRead, T> Reader<S> for ReadFixedSizeBuf<N, T> {
    type T = T;

    fn poll_reader(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut s: Pin<&mut S>,
    ) -> Poll<std::io::Result<Self::T>> {
        let mut buf = TokioReadBuf::new(&mut self.buf);
        loop {
            let rem = buf.remaining();
            if rem != 0 {
                ready!(s.as_mut().poll_read(cx, &mut buf))?;
                if buf.remaining() == rem {
                    return Err(eof()).into();
                }
            } else {
                return Poll::Ready(Ok((self.map)(self.buf)));
            }
        }
    }
}

pub struct ReadLenPrefixedBuf<T> {
    state: ReadBufState,
    map: fn(Vec<u8>) -> T,
}

enum ReadBufState {
    ReadingLenth([u8; 8], usize),
    ReadingBody(Vec<u8>),
    Done,
}

impl<T> ReadLenPrefixedBuf<T> {
    pub fn new(map: fn(Vec<u8>) -> T) -> Self {
        Self {
            state: ReadBufState::ReadingLenth([0; 8], 0),
            map,
        }
    }
}

impl<S: AsyncRead, T: Decode> Reader<S> for ReadLenPrefixedBuf<T> {
    type T = T;

    fn poll_reader(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut s: Pin<&mut S>,
    ) -> Poll<io::Result<Self::T>> {
        let buf = loop {
            match &mut self.state {
                ReadBufState::ReadingLenth(buf, cursor) => {
                    let mut buf = TokioReadBuf::new(&mut buf[*cursor..]);
                    loop {
                        ready!(s.as_mut().poll_read(cx, &mut buf))?;
                        if buf.remaining() == 0 {
                            break;
                        } else {
                            // A buffer will only ever return nothing if it's the end of the file.
                            return Err(eof()).into();
                        }
                    }

                    let buff = match mem::replace(&mut self.state, ReadBufState::Done) {
                        ReadBufState::ReadingLenth(b, _) => b,
                        _ => unreachable!(),
                    };

                    let len = u64::from_le_bytes(buff);
                    let len = len.try_into().unwrap(); // TODO: Error handling
                    let buf = vec![0; len]; // TODO: Can we avoid zeroing out the array cause it might help with performance???

                    self.state = ReadBufState::ReadingBody(buf);
                    continue;
                }
                ReadBufState::ReadingBody(b) => break b,
                ReadBufState::Done => panic!("Future polled after completion"),
            }
        };

        let mut buf = TokioReadBuf::new(buf);
        loop {
            let rem = buf.remaining();
            if rem != 0 {
                ready!(s.as_mut().poll_read(cx, &mut buf))?;
                if buf.remaining() == rem {
                    return Err(eof()).into();
                }
            } else {
                let buf = match std::mem::replace(&mut self.state, ReadBufState::Done) {
                    ReadBufState::ReadingBody(b) => b,
                    _ => unreachable!(),
                };

                return Poll::Ready(Ok((self.map)(buf)));
            }
        }
    }
}

pub struct ReadMap<T> {
    state: ReadMapState,
    items: usize,
    phantom: PhantomData<T>,
}

enum ReadMapState {
    ReadingLength([u8; 8], usize),
    // ReadingKey([u8; 8], Vec<u8>),
    // ReadingValue([u8; 8], Vec<u8>, Vec<u8>),
    // Done,
}

impl<T> ReadMap<T> {
    pub fn new() -> Self {
        Self {
            state: ReadMapState::ReadingLength([0; 8], 0),
            items: 0,
            phantom: PhantomData,
        }
    }
}

impl<S: AsyncRead, T> Reader<S> for ReadMap<T> {
    type T = T;

    fn poll_reader(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut s: Pin<&mut S>,
    ) -> Poll<io::Result<Self::T>> {
        // println!("POLL");
        // let mut this = self.as_mut().project();
        // loop {
        //     match this.state.as_mut().project() {
        //         ReadMapState::ReadingLength(mut buff, cursor) => {
        //             let mut buf = TokioReadBuf::new(&mut buff[*cursor..]);
        //             ready!(s.as_mut().poll_read(cx, &mut buf))?;
        //             if buf.remaining() == 0 {
        //                 let len = u64::from_le_bytes(buff);
        //                 let len = len.try_into().unwrap(); // TODO: Error handling

        //                 self.items = len;

        //                 println!("{:?}", len);

        //                 todo!();
        //             } else {
        //                 // A buffer will only ever return nothing if it's the end of the file.
        //                 return Err(eof()).into();
        //             }
        //         }
        //     }
        // }

        // let buf = loop {
        //     match &mut self.state {
        //         ReadBufState2::ReadingLenth(buf, cursor) => {
        //             let mut buf = TokioReadBuf::new(&mut buf[*cursor..]);
        //             loop {
        //                 ready!(s.as_mut().poll_read(cx, &mut buf))?;
        //                 if buf.remaining() == 0 {
        //                     break;
        //                 } else {
        //                     // A buffer will only ever return nothing if it's the end of the file.
        //                     return Err(eof()).into();
        //                 }
        //             }

        //             let buff = match mem::replace(&mut self.state, ReadBufState2::Done) {
        //                 ReadBufState2::ReadingLenth(b, _) => b,
        //                 _ => unreachable!(),
        //             };

        //             let len = u64::from_le_bytes(buff);
        //             let len = len.try_into().unwrap(); // TODO: Error handling
        //             let buf = vec![0; len]; // TODO: Can we avoid zeroing out the array cause it might help with performance???

        //             self.state = ReadBufState2::ReadingBody(buf);
        //             continue;
        //         }
        //         ReadBufState2::ReadingBody(b) => break b,
        //         ReadBufState2::Done => panic!("Future polled after completion"),
        //     }
        // };

        // let mut buf = TokioReadBuf::new(buf);
        // loop {
        //     let rem = buf.remaining();
        //     if rem != 0 {
        //         ready!(s.as_mut().poll_read(cx, &mut buf))?;
        //         if buf.remaining() == rem {
        //             return Err(eof()).into();
        //         }
        //     } else {
        //         let buf = match std::mem::replace(&mut self.state, ReadBufState2::Done) {
        //             ReadBufState2::ReadingBody(b) => b,
        //             _ => unreachable!(),
        //         };

        //         // return Poll::Ready(Ok((self.map)(buf)));
        //         todo!();
        //     }
        // }

        // TODO: Encode as Vec<(K, V)> and share existing writing code

        todo!();
    }
}
