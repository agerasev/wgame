use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Error, Result, parse2};

pub fn derive(input: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = parse2(input)?;

    let mod_ = quote! { wgame_shapes::attributes };
    let trait_ = quote! { #mod_::Attributes };
    let ident = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut code = quote! { #mod_::AttributeList::default() };
    match input.data {
        Data::Struct(data) => {
            for (i, field) in data.fields.into_iter().enumerate() {
                let ty = field.ty;
                let prefix = match field.ident {
                    Some(ident) => ident.to_string(),
                    None => format!("{i}"),
                };
                code = quote! {
                    #code
                    .chain(<#ty as #trait_>::attributes().with_prefix(#prefix))
                };
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
            fn attributes() -> #mod_::AttributeList {
                #code
            }
        }
    })
}
