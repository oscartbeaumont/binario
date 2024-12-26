use std::sync::Arc;

use binario::encode;
use bytes::{BufMut, BytesMut};
use futures::SinkExt;
use tokio::{
    io::{AsyncWriteExt, BufWriter},
    net::TcpStream,
};
use tokio_util::codec::{Encoder, Framed};

const BIG_STRING_LEN: usize = 1000;
const BIG_STRING_NETWORK_ITERS: usize = 500;

#[tokio::main]
async fn main() {
    bench_proto().await;
    bunch_bincode().await;
}

async fn bench_proto() {
    let string = Arc::new((0..BIG_STRING_LEN).map(|_| "X").collect::<String>());

    let conn = TcpStream::connect("127.0.0.1:9005").await.unwrap();
    let mut conn = BufWriter::new(conn);

    let before = std::time::Instant::now();

    for _ in 0..BIG_STRING_NETWORK_ITERS {
        encode(&string, &mut conn).await.unwrap();
    }

    let mid = std::time::Instant::now();
    let mid_before = mid - before;

    conn.flush().await.unwrap();

    let (total, flush) = (before.elapsed(), mid.elapsed());
    println!("Proto:");
    println!("Request: {:?}", mid_before);
    println!("Flush  : {:?}", flush);
    println!("        --------");
    println!("Total  : {:?}\n", total);
}

async fn bunch_bincode() {
    let string = Arc::new((0..BIG_STRING_LEN).map(|_| "X").collect::<String>());

    let conn = TcpStream::connect("127.0.0.1:9006").await.unwrap();
    let conn = BufWriter::new(conn);
    let mut conn = Framed::new(conn, BincodeEncoder {});

    let before = std::time::Instant::now();

    for _ in 0..BIG_STRING_NETWORK_ITERS {
        conn.send(&*string).await.unwrap();
    }

    let mid = std::time::Instant::now();
    let mid_before = mid - before;

    conn.flush().await.unwrap();

    let (total, flush) = (before.elapsed(), mid.elapsed());
    println!("Bincode:");
    println!("Request: {:?}", mid_before);
    println!("Flush  : {:?}", flush);
    println!("        --------");
    println!("Total  : {:?}\n", total);
}

pub struct BincodeEncoder;

impl<'a> Encoder<&'a str> for BincodeEncoder {
    type Error = bincode::Error;

    fn encode(&mut self, item: &'a str, dst: &mut BytesMut) -> Result<(), Self::Error> {
        bincode::serialize_into(dst.writer(), &item)
    }
}
