use std::{
    io::{self, Read},
    sync::atomic::{AtomicU32, Ordering},
};

use bytes::BytesMut;
use futures::StreamExt;
use tokio::{io::BufReader, net::TcpListener};
use tokio_util::codec::{Decoder, Framed};

static COUNTER: AtomicU32 = AtomicU32::new(1); // We start at one so the log output is correct

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:9006").await.unwrap();
    println!("Listening on http://127.0.0.1:9006");

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let stream = BufReader::new(stream);
        let mut stream = Framed::new(stream, BincodeDecoder {});
        tokio::task::spawn(async move {
            loop {
                while let Some(result) = stream.next().await {
                    match result.map_err(|v| *v) {
                        Ok(string) => {
                            println!("Reqs: {}", COUNTER.fetch_add(1, Ordering::Relaxed));
                            assert_eq!(string.len(), 1000);
                        }
                        Err(bincode::ErrorKind::Io(e))
                            if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                        {
                            println!("EOF"); // TODO: For some reason we get a stupid amount of EOF's

                            // assert_eq!(COUNTER.load(Ordering::Relaxed) - 1, 500);
                            // process::exit(0);
                            break;
                        }
                        Err(e) => {
                            panic!("Error: {}", e);
                        }
                    };
                }
            }
        });
    }
}

pub struct BincodeDecoder;

impl Decoder for BincodeDecoder {
    type Item = String;
    type Error = bincode::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if !buf.is_empty() {
            let mut reader = Reader::new(&buf[..]);
            let message = bincode::deserialize_from(&mut reader)?;
            let _ = buf.split_to(reader.amount());
            Ok(Some(message))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
struct Reader<'buf> {
    buf: &'buf [u8],
    amount: usize,
}

impl<'buf> Reader<'buf> {
    pub fn new(buf: &'buf [u8]) -> Self {
        Reader { buf, amount: 0 }
    }

    pub fn amount(&self) -> usize {
        self.amount
    }
}

impl<'buf, 'a> Read for &'a mut Reader<'buf> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.buf.read(buf)?;
        self.amount += bytes_read;
        Ok(bytes_read)
    }
}
