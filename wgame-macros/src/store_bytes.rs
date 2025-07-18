use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Error, Result, parse2};

pub fn derive(input: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = parse2(input)?;

    let mod_ = if let Some(attr) = input
        .attrs
        .into_iter()
        .find(|attr| attr.meta.path().is_ident("bytes_mod"))
    {
        attr.meta.require_list()?.tokens.clone()
    } else {
        quote! { wgame::gfx::bytes }
    };

    let trait_ = quote! { #mod_::StoreBytes };
    let ident = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut total_size = quote! { 0 };
    let mut store_code = quote! {};
    match input.data {
        Data::Struct(data) => {
            for (i, field) in data.fields.into_iter().enumerate() {
                let ty = field.ty;
                total_size = quote! { #total_size + <#ty as #trait_>::SIZE };

                let expr = match field.ident {
                    Some(ident) => quote! { self.#ident },
                    None => quote! { self.#i },
                };
                store_code = quote! {
                    #store_code
                    <#ty as #trait_>::store_bytes(&#expr, dst);
                }
            }
        }
        Data::Union(_) | Data::Enum(_) => {
            return Err(Error::new(
                Span::call_site(),
                "Enums and unions are not supported",
            ));
        }
    };

    Ok(quote! {
        impl #impl_generics #trait_ for #ident #ty_generics #where_clause {
            const SIZE: usize = #total_size;

            fn store_bytes<D: #mod_::BytesSink>(&self, dst: &mut D) {
                #store_code
            }
        }
    })
}
