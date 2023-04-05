use std::io::Cursor;

use binario::{decode, encode, Decode, Encode};

#[derive(Debug, Encode, Decode, PartialEq, Eq)]
pub struct MyMessage {
    // pub a: i32, // TODO: Impl for primitive
    pub b: String,
    // pub c: Vec<u8>, // TODO: Impl for primitive
    // pub d: HashMap<String, String>, // TODO: Impl for primitive
}

// TODO: Support for generics and generic bounds

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
            // c: vec![],
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

    // Test max length
    {
        let msg = (0..10000).map(|_| "X").collect::<String>();
        let mut buf = Vec::new();
        encode(&msg, &mut buf).await.unwrap();
        assert_eq!(buf.len(), 8 /* Buf len */ + 10000);

        let buf = Cursor::new(buf);
        let msg2: String = decode(buf).await.unwrap();
        assert_eq!(msg, msg2);
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
