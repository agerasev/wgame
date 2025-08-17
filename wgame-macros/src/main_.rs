use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Error, Ident, ItemFn, Result, parse2, spanned::Spanned};

fn impl_(attr: TokenStream, item: TokenStream, run: Ident) -> Result<TokenStream> {
    if !attr.is_empty() {
        return Err(Error::new(attr.span(), "No attributes expected"));
    }

    let mut amain = parse2::<ItemFn>(item)?;
    let main = amain.sig.ident;
    let ident = Ident::new("__wgame_main", main.span());
    amain.sig.ident = ident.clone();

    Ok(quote! {
        #amain
        wgame::#run!(#main, #ident);
    })
}

pub fn impl_app(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    impl_(attr, item, Ident::new("run_app", Span::call_site()))
}

pub fn impl_window(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    impl_(attr, item, Ident::new("run_window", Span::call_site()))
}
