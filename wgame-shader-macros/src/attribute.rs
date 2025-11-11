use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Error, Result, parse2};

pub fn derive(input: TokenStream, mod_: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = parse2(input)?;

    let trait_ = quote! { #mod_::Attribute };
    let ident = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut bindings = quote! { #mod_::BindingList::default() };
    let mut total_size = quote! { 0 };
    let mut store_code = quote! {};
    match input.data {
        Data::Struct(data) => {
            for (i, field) in data.fields.into_iter().enumerate() {
                let ty = field.ty;

                let (prefix, expr) = match field.ident {
                    Some(ident) => (ident.to_string(), quote! { self.#ident }),
                    None => (format!("{i}"), quote! { self.#i }),
                };

                bindings = quote! {
                    #bindings
                    .chain(<#ty as #trait_>::bindings().with_prefix(#prefix))
                };

                total_size = quote! { #total_size + <#ty as #trait_>::SIZE };

                store_code = quote! {
                    #store_code
                    <#ty as #trait_>::store(&#expr, dst);
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
            fn bindings() -> #mod_::BindingList {
                #bindings
            }

            const SIZE: usize = #total_size;

            fn store(&self, dst: &mut #mod_::BytesSink) {
                #store_code
            }
        }
    })
}
