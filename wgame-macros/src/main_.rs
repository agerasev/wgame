use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Error, Ident, ItemFn, MetaNameValue, Result, Token, parse::Parser, parse2,
    punctuated::Punctuated, spanned::Spanned,
};

fn impl_(item: TokenStream, run_macro: Ident, args: TokenStream) -> Result<TokenStream> {
    let mut amain = parse2::<ItemFn>(item)?;
    let main = amain.sig.ident;
    let ident = Ident::new("__wgame_main", main.span());
    amain.sig.ident = ident.clone();

    Ok(quote! {
        #amain
        wgame::#run_macro!(#main, #ident, #args);
    })
}

pub fn impl_app(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    if !attr.is_empty() {
        return Err(Error::new(attr.span(), "No attributes expected"));
    }
    impl_(item, Ident::new("run_app", Span::call_site()), quote! {})
}

pub fn impl_window(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let mut config = quote! { wgame::WindowConfig::default() };
    let parser = Punctuated::<MetaNameValue, Token![,]>::parse_terminated;
    for MetaNameValue { path, value, .. } in parser.parse2(attr)? {
        let key = path
            .get_ident()
            .ok_or_else(|| Error::new(path.span(), "Bad key"))?;
        config = quote! { #config.#key(#value) };
    }

    impl_(item, Ident::new("run_window", Span::call_site()), config)
}
