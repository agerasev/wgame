use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Error, Ident, ItemFn, Result, parse2, spanned::Spanned};

#[proc_macro_attribute]
pub fn main(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match main_impl(attr.into(), item.into()) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}

fn main_impl(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    if !attr.is_empty() {
        return Err(Error::new(attr.span(), "No attributes expected"));
    }

    let mut amain = parse2::<ItemFn>(item)?;
    if amain.sig.ident != "main" {
        return Err(Error::new(
            amain.sig.ident.span(),
            "Main function name must be `main`",
        ));
    }
    let ident = Ident::new("__wgame_async_main", amain.sig.ident.span());
    amain.sig.ident = ident.clone();

    Ok(quote! {
        #amain
        wgame::run_main!(#ident);
    })
}
