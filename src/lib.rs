//! TODO

mod decode;
mod decode_impls;
mod decode_utils;
mod encode;
mod encode_impls;
mod encode_utils;

pub use decode::*;
pub use decode_impls::*;
pub use decode_utils::*;
pub use encode::*;
pub use encode_impls::*;
pub use encode_utils::*;

pub use binario_derive::*;

#[doc(hidden)]
pub mod internal {
    // Renamed exports to try and make it not show up in Rust Analyzer autocomplete

    pub use pin_project::pin_project as binario_internal_pin_project;
    pub use tokio::io::{AsyncRead as BinarioAsyncRead, AsyncWrite as BinarioAsyncWrite};
}
