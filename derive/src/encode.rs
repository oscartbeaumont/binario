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

    let (writer_init, writer_fields, writer_impl, byte_len_impl) = match data {
        Data::Struct(data) => {
            let fields_init = data.fields.iter().map(|field| {
                let name = &field.ident;
                let ty = &field.ty;

                quote! {
                    #name: #crate_name::WriterOrDone::Writer(<#ty as Encode>::encode(&self.#name)),
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
                        #name: #crate_name::WriterOrDone<'a, #ty>,
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

            let byte_len_impl = data
                .fields
                .iter()
                .map(|field| {
                    let name = &field.ident;
                    let ty = &field.ty;

                    quote! {
                        + <#ty as Encode>::byte_len(&self.#name)
                    }
                })
                .collect::<TokenStream>();

            (fields_init, fields, impls, byte_len_impl)
        }
        Data::Enum(_) => todo!("Enum's are not supported yet!"),
        Data::Union(_) => todo!("Union's are not supported yet!"),
    };

    let encode_impl_header = quote!(impl #crate_name::Encode for #ident); // TODO: Support generics and custom bounds
    let writer_header = quote!(struct CustomWriter<'a>); // TODO: Support generics and custom bounds
    let writer_impl_header = quote!(impl<'a> #crate_name::Writer for CustomWriter<'a>); // TODO: Support generics and custom bounds

    Ok(quote! {
        const _: () = {
            #[automatically_derived]
            #encode_impl_header {
                type Writer<'a> = CustomWriter<'a>
                where
                    Self: 'a;

                fn byte_len(&self) -> usize {
                    0 #byte_len_impl
                }

                fn encode<'a>(&'a self) -> Self::Writer<'a> {
                    CustomWriter {
                        #writer_init
                    }
                }
            }

            #[automatically_derived]
            #[pin_project::pin_project(project = CustomWriterProj)]
            pub #writer_header {
                #writer_fields
            }

            #writer_impl_header {
                fn poll_writer<S: #crate_name::internal::BinarioAsyncWrite>(
                    self: std::pin::Pin<&mut Self>,
                    cx: &mut std::task::Context<'_>,
                    mut s: std::pin::Pin<&mut S>,
                ) -> std::task::Poll<std::io::Result<()>> {
                    let this = self.project();

                    #writer_impl

                    std::task::Poll::Ready(Ok(()))
                }
            }
        };
    }
    .into())
}
