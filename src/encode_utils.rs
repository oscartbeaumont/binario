use std::{
    io, mem,
    pin::{pin, Pin},
    task::{Context, Poll},
};

use pin_project::pin_project;
use tokio::io::AsyncWrite;

use crate::{Encode, Writer};

/// TODO
pub struct WriteFixedBuf<const N: usize> {
    buf: [u8; N],
    cursor: usize,
}

impl<const N: usize> WriteFixedBuf<N> {
    pub fn new(buf: [u8; N]) -> Self {
        Self { buf, cursor: 0 }
    }
}

impl<const N: usize> Writer for WriteFixedBuf<N> {
    fn poll_writer<S: AsyncWrite>(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut s: Pin<&mut S>,
    ) -> Poll<io::Result<()>> {
        // loop {
        //     match s
        //         .as_mut()
        //         .poll_write(cx, &self.buf[self.cursor..self.buf.byte_len()])
        //     {
        //         Poll::Ready(Ok(n)) => {
        //             self.cursor += n;

        //             if n == 0 {
        //                 return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
        //             } else if self.cursor == self.buf.byte_len() {
        //                 return Poll::Ready(Ok(()));
        //             }
        //         }
        //         Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
        //         Poll::Pending => return Poll::Pending,
        //     }
        // }
        todo!();
    }
}

/// TODO
pub struct WriteBuf<'a, T: Encode> {
    len_buf: [u8; 8],
    len_cursor: usize,
    buf: &'a [T],
    cursor: usize,
}

impl<'a, T: Encode> WriteBuf<'a, T> {
    pub fn new(len: usize, buf: &'a [T]) -> Self {
        let len: u64 = len.try_into().unwrap();
        Self {
            len_buf: len.to_le_bytes(), // x86_64 uses little endian so we gonna stick with it
            len_cursor: 0,
            buf,
            cursor: 0,
        }
    }
}

impl<'a, T: Encode> Writer for WriteBuf<'a, T> {
    fn poll_writer<S: AsyncWrite>(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut s: Pin<&mut S>,
    ) -> Poll<io::Result<()>> {
        loop {
            if self.len_cursor == 8 {
                break;
            }

            match s.as_mut().poll_write(cx, &self.len_buf) {
                Poll::Ready(Ok(n)) => {
                    self.len_cursor += n;
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        loop {
            // TODO: Optimised encode method that works on `Vec<T>` where `T: Encode` so that we don't need `WriteBuf2`
            let w = self.buf[self.cursor].encode(); // TODO: Is the fact that this is recreating the writer every time a problem when it holds state?
            let w = pin!(w);

            match w.poll_writer(cx, s.as_mut()) {
                Poll::Ready(Ok(_)) => {
                    self.cursor += 1;

                    if self.cursor == self.buf.len() {
                        return Poll::Ready(Ok(()));
                    }
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// TODO
pub struct WriteBuf2<'a> {
    len_buf: [u8; 8],
    len_cursor: usize,
    buf: &'a [u8],
    cursor: usize,
}

impl<'a> WriteBuf2<'a> {
    pub fn new(len: usize, buf: &'a [u8]) -> Self {
        let len: u64 = len.try_into().unwrap();
        Self {
            len_buf: len.to_le_bytes(), // x86_64 uses little endian so we gonna stick with it
            len_cursor: 0,
            buf,
            cursor: 0,
        }
    }
}

impl<'a> Writer for WriteBuf2<'a> {
    fn poll_writer<S: AsyncWrite>(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut s: Pin<&mut S>,
    ) -> Poll<io::Result<()>> {
        loop {
            if self.len_cursor == 8 {
                break;
            }

            match s.as_mut().poll_write(cx, &self.len_buf) {
                Poll::Ready(Ok(n)) => {
                    self.len_cursor += n;
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        loop {
            match s
                .as_mut()
                .poll_write(cx, &self.buf[self.cursor..self.buf.len()])
            {
                Poll::Ready(Ok(n)) => {
                    self.cursor += n;

                    if n == 0 {
                        return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
                    } else if self.cursor == self.buf.len() {
                        return Poll::Ready(Ok(()));
                    }
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

// /// TODO
// #[pin_project(project = WriteMapProj)]
// pub struct WriteMap<'a, I: Iterator<Item = (&'a K, &'a V)>, K: Encode + 'a, V: Encode + 'a> {
//     #[pin]
//     state: WriteMapState<'a, K, V>,
//     buf: I,
//     cursor: usize,
// }

// #[pin_project(project = WriteMapStateProj)]
// pub enum WriteMapState<'a, K: Encode + 'a, V: Encode + 'a> {
//     WriteMapLen([u8; 8], usize),
//     WriteKey(
//         [u8; 8],
//         usize,
//         #[pin] K::Writer<'a>,
//         Option<([u8; 8], V::Writer<'a>)>,
//     ),
//     WriteValue([u8; 8], usize, #[pin] V::Writer<'a>),
//     Done,
// }

// impl<'a, I: Iterator<Item = (&'a K, &'a V)>, K: Encode, V: Encode> WriteMap<'a, I, K, V> {
//     pub fn new(len: usize, buf: I) -> Self {
//         let len: u64 = len.try_into().unwrap();
//         Self {
//             state: WriteMapState::WriteMapLen(len.to_le_bytes(), 0), // x86_64 uses little endian so we gonna stick with it
//             buf,
//             cursor: 0,
//         }
//     }
// }

// impl<'a, I: Iterator<Item = (&'a K, &'a V)> + Unpin, K: Encode, V: Encode> Writer
//     for WriteMap<'a, I, K, V>
// {
//     fn poll_writer<S: AsyncWrite>(
//         mut self: Pin<&mut Self>,
//         cx: &mut Context<'_>,
//         mut s: Pin<&mut S>,
//     ) -> Poll<io::Result<()>> {
//         let mut this = self.as_mut().project();
//         loop {
//             match this.state.as_mut().project() {
//                 WriteMapStateProj::WriteMapLen(buf, cursor) => loop {
//                     if *cursor == 8 {
//                         let (k, v) = this.buf.next().unwrap(); // TODO: Handle 'None' here
//                         let k_len = k.byte_len();
//                         let k_len: u64 = k_len.try_into().unwrap();
//                         let k_len = k_len.to_le_bytes();
//                         let k_writer: K::Writer<'_> = k.encode();

//                         let v_len = v.byte_len();
//                         let v_len: u64 = v_len.try_into().unwrap();
//                         let v_len = v_len.to_le_bytes();
//                         let v_writer: V::Writer<'_> = v.encode();

//                         this.state.set(WriteMapState::WriteKey(
//                             k_len,
//                             0,
//                             k_writer,
//                             Some((v_len, v_writer)),
//                         ));
//                         break;
//                     }

//                     match s.as_mut().poll_write(cx, buf) {
//                         Poll::Ready(Ok(n)) => {
//                             *cursor += n;
//                         }
//                         Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
//                         Poll::Pending => return Poll::Pending,
//                     }
//                 },
//                 WriteMapStateProj::WriteKey(buf, cursor, k_writer, done) => {
//                     loop {
//                         if *cursor == 8 {
//                             break;
//                         }

//                         match s.as_mut().poll_write(cx, buf) {
//                             Poll::Ready(Ok(n)) => {
//                                 *cursor += n;
//                             }
//                             Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
//                             Poll::Pending => return Poll::Pending,
//                         }
//                     }

//                     match k_writer.poll_writer(cx, s.as_mut()) {
//                         Poll::Ready(Ok(())) => {
//                             let (buf, writer) = done.take().expect("unreachable");
//                             this.state.set(WriteMapState::WriteValue(buf, 0, writer));
//                             continue;
//                         }
//                         Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
//                         Poll::Pending => return Poll::Pending,
//                     }
//                 }
//                 WriteMapStateProj::WriteValue(buf, cursor, v_writer) => {
//                     loop {
//                         if *cursor == 8 {
//                             break;
//                         }

//                         match s.as_mut().poll_write(cx, buf) {
//                             Poll::Ready(Ok(n)) => {
//                                 *cursor += n;
//                             }
//                             Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
//                             Poll::Pending => return Poll::Pending,
//                         }
//                     }

//                     match v_writer.poll_writer(cx, s.as_mut()) {
//                         Poll::Ready(Ok(())) => {
//                             // TODO: Deduplicate this code higher up

//                             let (k, v) = match this.buf.next() {
//                                 Some(v) => v,
//                                 None => {
//                                     return Poll::Ready(Ok(()));
//                                 }
//                             };

//                             let k_len = k.byte_len();
//                             let k_len: u64 = k_len.try_into().unwrap();
//                             let k_len = k_len.to_le_bytes();
//                             let k_writer: K::Writer<'_> = k.encode();

//                             let v_len = v.byte_len();
//                             let v_len: u64 = v_len.try_into().unwrap();
//                             let v_len = v_len.to_le_bytes();
//                             let v_writer: V::Writer<'_> = v.encode();

//                             // cursor += 1;
//                             this.state.set(WriteMapState::WriteKey(
//                                 k_len,
//                                 0,
//                                 k_writer,
//                                 Some((v_len, v_writer)),
//                             ));
//                         }
//                         Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
//                         Poll::Pending => return Poll::Pending,
//                     }
//                 }
//                 WriteMapStateProj::Done => {
//                     unreachable!("'WriteMapState' future polled after completion")
//                 }
//             }
//         }
//     }
// }

/// TODO
#[pin_project(project = WriteMapProj)]
pub struct WriteMap<'a, I: Iterator<Item = (&'a K, &'a V)>, K: Encode + 'a, V: Encode + 'a> {
    #[pin]
    state: WriteMapState<'a, K, V>,
    buf: I,
    cursor: usize,
}

#[pin_project(project = WriteMapStateProj)]
pub enum WriteMapState<'a, K: Encode + 'a, V: Encode + 'a> {
    WriteMapLen([u8; 8], usize),
    WriteKey(
        [u8; 8],
        usize,
        #[pin] K::Writer<'a>,
        Option<([u8; 8], V::Writer<'a>)>,
    ),
    WriteValue([u8; 8], usize, #[pin] V::Writer<'a>),
    Done,
}

impl<'a, I: Iterator<Item = (&'a K, &'a V)>, K: Encode, V: Encode> WriteMap<'a, I, K, V> {
    pub fn new(len: usize, buf: I) -> Self {
        let len: u64 = len.try_into().unwrap();
        Self {
            state: WriteMapState::WriteMapLen(len.to_le_bytes(), 0), // x86_64 uses little endian so we gonna stick with it
            buf,
            cursor: 0,
        }
    }
}

impl<'a, I: Iterator<Item = (&'a K, &'a V)> + Unpin, K: Encode, V: Encode> Writer
    for WriteMap<'a, I, K, V>
{
    fn poll_writer<S: AsyncWrite>(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut s: Pin<&mut S>,
    ) -> Poll<io::Result<()>> {
        todo!();
    }
}
