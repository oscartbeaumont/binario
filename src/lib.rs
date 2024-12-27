//! An experimental asynchronous zero-fluff binary encoder / decoder
//!
//! This is very similar to [bincode](https://github.com/bincode-org/bincode) however it has been designed from the ground up to work with [Tokio](https://tokio.rs)'s [AsyncRead](https://docs.rs/tokio/latest/tokio/io/trait.AsyncRead.html) [AsyncWrite](https://docs.rs/tokio/latest/tokio/io/trait.AsyncWrite.html) traits. This allows it to perform better when used with [TcpStream](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html)'s than an equivalent bincode program.
//!
//! **Benchmarking is still in progress but so far I am seeing about a 3 times performance win over bincode when used over a TCP connection. Binario does and will likely will continue to loose on in-memory head to head performance due to the overhead of async.**

mod decode;
mod decode_impls;
mod decode_utils;
mod encode;
mod encode_impls;

pub use decode::*;
pub use decode_impls::*;
pub use decode_utils::*;
pub use encode::*;

pub use binario_derive::*;

#[doc(hidden)]
pub mod internal {
    // Renamed exports to try and make it not show up in Rust Analyzer autocomplete

    pub use pin_project::pin_project as binario_internal_pin_project;
    pub use tokio::io::{AsyncRead as BinarioAsyncRead, AsyncWrite as BinarioAsyncWrite};
}
