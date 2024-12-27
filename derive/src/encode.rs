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

    let (encode_impl, byte_len_impl) = match data {
        Data::Struct(data) => {
            let encode_impl = data
                .fields
                .iter()
                .map(|field| {
                    let ident = &field.ident;

                    quote!(<_ as #crate_name::Encode>::encode(&self.#ident, s.as_mut()).await?;)
                })
                .collect::<TokenStream>();

            let byte_len_impl = data
                .fields
                .iter()
                .map(|field| {
                    let name = &field.ident;
                    let ty = &field.ty;

                    quote! {
                        + <#ty as #crate_name::Encode>::byte_len(&self.#name)
                    }
                })
                .collect::<TokenStream>();

            (encode_impl, byte_len_impl)
        }
        Data::Enum(_) => todo!("Enum's are not supported yet!"),
        Data::Union(_) => todo!("Union's are not supported yet!"),
    };

    let encode_impl_header = quote!(impl #crate_name::Encode for #ident); // TODO: Support generics and custom bounds

    Ok(quote! {
        const _: () = {
            #[automatically_derived]
            #encode_impl_header {
                async fn encode<S: #crate_name::internal::BinarioAsyncWrite>(&self, mut s: std::pin::Pin<&mut S>) -> std::io::Result<()> {
                    #encode_impl
                    Ok(())
                }

                fn byte_len(&self) -> usize {
                    0 #byte_len_impl
                }
            }
        };
    }
    .into())
}
