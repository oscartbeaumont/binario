use quote::{__private::TokenStream, quote};
use syn::{parse, Data, DeriveInput};

pub fn derive(input: proc_macro::TokenStream) -> syn::Result<proc_macro::TokenStream> {
    let DeriveInput {
        ident,
        generics,
        data,
        attrs,
        ..
    } = parse::<DeriveInput>(input)?;

    let crate_name = quote!(binario);

    let fields_decode_impl = match data {
        Data::Struct(data) => {
            // let fields_init = data.fields.iter().map(|field| {
            //     let name = &field.ident;
            //     let ty = &field.ty;

            //     quote! {
            //         #name: #crate_name::ValueOrReader::Reader(<#ty as #crate_name::Decode>::decode::<S>()),
            //     }
            // }).collect::<TokenStream>();

            data.fields
                .iter()
                .map(|field| {
                    let name = &field.ident;
                    let ty = &field.ty;

                    quote! {
                        #name: <#ty as #crate_name::Decode>::decode(s.as_mut()).await?,
                    }
                })
                .collect::<TokenStream>()
        }
        Data::Enum(_) => todo!("Enum's are not supported yet!"),
        Data::Union(_) => todo!("Union's are not supported yet!"),
    };

    let decode_impl_header = quote!(impl #crate_name::Decode for #ident); // TODO: Support generics and custom bounds

    // let reader_header = quote!(struct CustomReader<S: #crate_name::internal::BinarioAsyncRead>); // TODO: Support generics and custom bounds
    // let reader_impl_header = quote!(impl<S: #crate_name::internal::BinarioAsyncRead> #crate_name::Reader<S> for CustomReader<S>); // TODO: Support generics and custom bounds

    Ok(quote! {
        const _: () = {
            #[automatically_derived]
            #decode_impl_header {
                async fn decode<S: binario::internal::BinarioAsyncRead>(mut s: std::pin::Pin<&mut S>) -> std::io::Result<Self> {
                    Ok(Self {
                        #fields_decode_impl
                    })
                }
            }
        };
    }
    .into())
}
