use std::{
    process,
    sync::atomic::{AtomicU32, Ordering},
};

use binario::decode;
use tokio::{io::BufReader, net::TcpListener};

static COUNTER: AtomicU32 = AtomicU32::new(1); // We start at one so the log output is correct

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:9005").await.unwrap();
    println!("Listening on http://127.0.0.1:9005");

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let mut stream = BufReader::new(stream);
        tokio::task::spawn(async move {
            loop {
                match decode::<String, _>(&mut stream).await {
                    Ok(string) => {
                        println!("Reqs: {}", COUNTER.fetch_add(1, Ordering::Relaxed));
                        assert_eq!(string.len(), 245);
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                        assert_eq!(COUNTER.load(Ordering::Relaxed) - 1, 1000);
                        println!("Done");
                        process::exit(0);
                    }
                    Err(e) => {
                        panic!("Error: {}", e);
                    }
                };
            }
        });
    }
}
