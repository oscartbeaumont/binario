//! TODO

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Encode)]
pub fn encode_derive(_item: TokenStream) -> TokenStream {
    quote! {
        // TODO: Trait impl
    }
    .into()
}

#[proc_macro_derive(Decode)]
pub fn decode_derive(_item: TokenStream) -> TokenStream {
    quote! {
        // TODO: Trait impl
    }
    .into()
}
