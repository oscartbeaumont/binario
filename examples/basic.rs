use std::{io::Cursor, task::Poll};

use binario::{decode, encode, Decode, Encode};

#[derive(Debug, Encode, Decode, PartialEq, Eq)]
pub struct MyMessage {
    // pub a: i32, // TODO
    pub b: String,
    pub c: Vec<u8>,
}

// TODO: rexport 'AsyncWrite' and pin project from `lib.rs`
const _: () = {
    impl Encode for MyMessage {
        type Writer<'a, S: tokio::io::AsyncWrite + 'a> = MyMessageWriter<'a, S>
        where
            Self: 'a;

        fn encode<'a, S: tokio::io::AsyncWrite + 'a>(&'a self) -> Self::Writer<'a, S> {
            MyMessageWriter {
                b: binario::WriterOrDone::Writer(<String as Encode>::encode::<S>(&self.b)),
                c: binario::WriterOrDone::Writer(<Vec<u8> as Encode>::encode::<S>(&self.c)),
            }
        }
    }

    // TODO: Use pin project lite
    #[pin_project::pin_project(project = MyMessageWriterProj)]
    pub struct MyMessageWriter<'a, S: tokio::io::AsyncWrite> {
        #[pin]
        b: binario::WriterOrDone<'a, String, S>,
        #[pin]
        c: binario::WriterOrDone<'a, Vec<u8>, S>,
    }

    impl<'a, S: tokio::io::AsyncWrite> binario::Writer<S> for MyMessageWriter<'a, S> {
        fn poll_writer(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            mut s: std::pin::Pin<&mut S>,
        ) -> std::task::Poll<std::io::Result<()>> {
            let this = self.project();

            match this.b.unsafe_poll(cx, s.as_mut()) {
                Some(result) => return result,
                None => {}
            }

            // match this.c.unsafe_poll(cx, s.as_mut()) {
            //     Some(result) => return result,
            //     None => {}
            // }

            return Poll::Ready(Ok(()));
        }
    }

    impl binario::Decode for MyMessage {
        type Reader<S: tokio::io::AsyncRead> = MyMessageReader<S>;

        fn decode<S: tokio::io::AsyncRead>() -> Self::Reader<S> {
            MyMessageReader {
                b: binario::ValueOrReader::Reader(<String as binario::Decode>::decode::<S>()),
                // c: binario::ValueOrReader::Reader(<Vec<u8> as binario::Decode>::decode::<S>()),
            }
        }
    }

    #[pin_project::pin_project(project = MyMessageReaderProj)]
    pub struct MyMessageReader<S: tokio::io::AsyncRead> {
        #[pin]
        b: binario::ValueOrReader<String, S>,
        // #[pin]
        // c: binario::ValueOrReader<Vec<u8>, S>,
    }

    impl<S: tokio::io::AsyncRead> binario::Reader<S> for MyMessageReader<S> {
        type T = MyMessage;

        fn poll_reader(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            mut s: std::pin::Pin<&mut S>,
        ) -> Poll<std::io::Result<Self::T>> {
            let this = self.as_mut().project();

            match this.b.unsafe_poll(cx, s.as_mut()) {
                Some(result) => return result,
                None => {}
            }

            // match this.c.unsafe_poll(cx, s.as_mut()) {
            //     Some(result) => return result,
            //     None => {}
            // }

            Poll::Ready(Ok(MyMessage {
                b: self.b.unsafe_take(),
                c: Default::default(),
            }))
        }
    }
};

// TODO: Macro support for enums
// #[derive(Encode, Decode)]
// pub enum MyEnum {
//     // #[proto(tag = 1)]
//     A,
//     // #[proto(tag = 2)]
//     B,
//     // #[proto(tag = 3)]
//     C,
// }

#[tokio::main]
async fn main() {
    {
        let msg = MyMessage {
            b: "abc".to_string(),
            c: vec![],
        };
        let mut buf = Vec::new();
        encode(&msg, &mut buf).await.unwrap();
        println!("{:?}", buf);

        let buf = Cursor::new(buf);
        let msg2: MyMessage = decode(buf).await.unwrap();
        assert_eq!(msg, msg2);
        println!("{:?}\n", msg2);
    }

    {
        let msg = 42u8;
        let mut buf = Vec::new();
        encode(&msg, &mut buf).await.unwrap();
        println!("{:?}", buf);

        let buf = Cursor::new(buf);
        let msg2: u8 = decode(buf).await.unwrap();
        assert_eq!(msg, msg2);
        println!("{:?}\n", msg2);
    }

    {
        let msg = "abc".to_string();
        let mut buf = Vec::new();
        encode(&msg, &mut buf).await.unwrap();
        println!("{:?}", buf);

        let buf = Cursor::new(buf);
        let msg2: String = decode(buf).await.unwrap();
        assert_eq!(msg, msg2);
        println!("{:?}\n", msg2);
    }

    let msg = vec![1; 5];
    let mut buf = Vec::new();
    encode(&msg, &mut buf).await.unwrap();
    println!("{:?}", buf);

    let msg = vec![2; 5];
    let mut buf = Vec::new();
    encode(&msg, &mut buf).await.unwrap();
    println!("{:?}", buf);

    let msg = &[1, 2, 3, 4];
    let mut buf = Vec::new();
    encode(&msg, &mut buf).await.unwrap();
    println!("{:?}", buf);

    // let msg = MyMessage { a: 42 };
    // let mut buf = Vec::new();
    // encode(&msg, &mut buf).await.unwrap();
    // println!("{:?}", buf);
}
