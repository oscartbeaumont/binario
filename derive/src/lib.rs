//! TODO

use proc_macro::TokenStream;

mod decode;
mod encode;

#[proc_macro_derive(Encode, attributes(binario))]
pub fn encode_derive(input: TokenStream) -> TokenStream {
    encode::derive(input).unwrap_or_else(|err| err.into_compile_error().into())
}

#[proc_macro_derive(Decode, attributes(binario))]
pub fn decode_derive(input: TokenStream) -> TokenStream {
    decode::derive(input).unwrap_or_else(|err| err.into_compile_error().into())
}
