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

    let (writer_init, writer_fields, writer_impl) = match data {
        Data::Struct(data) => {
            let fields_init = data.fields.iter().map(|field| {
                let name = &field.ident;
                let ty = &field.ty;

                quote! {
                    #name: #crate_name::WriterOrDone::Writer(<#ty as Encode>::encode::<S>(&self.#name)),
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
                        #name: #crate_name::WriterOrDone<'a, #ty, S>,
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

            (fields_init, fields, impls)
        }
        Data::Enum(_) => todo!("Enum's are not supported yet!"),
        Data::Union(_) => todo!("Union's are not supported yet!"),
    };

    let encode_impl_header = quote!(impl #crate_name::Encode for #ident); // TODO: Support generics and custom bounds
    let writer_header =
        quote!(struct CustomWriter<'a, S: #crate_name::internal::BinarioAsyncWrite>); // TODO: Support generics and custom bounds
    let writer_impl_header = quote!(impl<'a, S: #crate_name::internal::BinarioAsyncWrite> #crate_name::Writer<S> for CustomWriter<'a, S>); // TODO: Support generics and custom bounds

    Ok(quote! {
        const _: () = {
            #[automatically_derived]
            #encode_impl_header {
                type Writer<'a, S: #crate_name::internal::BinarioAsyncWrite + 'a> = CustomWriter<'a, S>
                where
                    Self: 'a;

                fn encode<'a, S: #crate_name::internal::BinarioAsyncWrite + 'a>(&'a self) -> Self::Writer<'a, S> {
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
                fn poll_writer(
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
