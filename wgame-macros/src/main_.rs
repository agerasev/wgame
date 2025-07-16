use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Ident, ItemFn, Result, parse2, spanned::Spanned};

pub fn impl_(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    if !attr.is_empty() {
        return Err(Error::new(attr.span(), "No attributes expected"));
    }

    let mut amain = parse2::<ItemFn>(item)?;
    let main = amain.sig.ident;
    let ident = Ident::new("__wgame_async_main", main.span());
    amain.sig.ident = ident.clone();

    Ok(quote! {
        #amain
        wgame::run!(#main, #ident);
    })
}
