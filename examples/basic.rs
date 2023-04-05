use std::io::Cursor;

use binario::{decode, encode, Encode};

#[derive(Encode)]
pub struct MyMessage {
    pub a: i32,
    // pub b: String,
}

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
