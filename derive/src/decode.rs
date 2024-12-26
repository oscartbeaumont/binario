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

    let (reader_init, reader_fields, reader_impl, reader_return_init) = match data {
        Data::Struct(data) => {
            let fields_init = data.fields.iter().map(|field| {
                let name = &field.ident;
                let ty = &field.ty;

                quote! {
                    #name: #crate_name::ValueOrReader::Reader(<#ty as #crate_name::Decode>::decode::<S>()),
                }
            }).collect::<TokenStream>();

            let fields = data
                .fields
                .iter()
                .map(|field| {
                    let name = &field.ident;
                    let ty = &field.ty;

                    quote! {
                        #[pin]
                        #name: #crate_name::ValueOrReader<#ty, S>,
                    }
                })
                .collect::<TokenStream>();

            let impls = data
                .fields
                .iter()
                .map(|field| {
                    let name = &field.ident;

                    quote! {
                        match this.#name.unsafe_poll(cx, s.as_mut()) {
                            Some(result) => return result,
                            None => {}
                        }
                    }
                })
                .collect::<TokenStream>();

            let return_init = data
                .fields
                .iter()
                .map(|field| {
                    let name = &field.ident;

                    quote! {
                        #name: self.#name.unsafe_take(),
                    }
                })
                .collect::<TokenStream>();

            (fields_init, fields, impls, return_init)
        }
        Data::Enum(_) => todo!("Enum's are not supported yet!"),
        Data::Union(_) => todo!("Union's are not supported yet!"),
    };

    let decode_impl_header = quote!(impl #crate_name::Decode for #ident); // TODO: Support generics and custom bounds
    let reader_header = quote!(struct CustomReader<S: #crate_name::internal::BinarioAsyncRead>); // TODO: Support generics and custom bounds
    let reader_impl_header = quote!(impl<S: #crate_name::internal::BinarioAsyncRead> #crate_name::Reader<S> for CustomReader<S>); // TODO: Support generics and custom bounds

    Ok(quote! {
        const _: () = {
            #[automatically_derived]
            #decode_impl_header {
                type Reader<S: #crate_name::internal::BinarioAsyncRead> = CustomReader<S>;

                fn decode<S: #crate_name::internal::BinarioAsyncRead>() -> Self::Reader<S> {
                    CustomReader {
                        #reader_init
                    }
                }
            }

            #[automatically_derived]
            #[pin_project::pin_project(project = CustomReaderProj)]
            pub #reader_header {
                #reader_fields
            }

            #reader_impl_header {
                type T = #ident;

                fn poll_reader(
                    mut self: std::pin::Pin<&mut Self>,
                    cx: &mut std::task::Context<'_>,
                    mut s: std::pin::Pin<&mut S>,
                ) -> std::task::Poll<std::io::Result<Self::T>> {
                    let mut this = self.as_mut().project();

                    #reader_impl

                    std::task::Poll::Ready(Ok(#ident {
                        #reader_return_init
                    }))
                }
            }
        };
    }
    .into())
}
