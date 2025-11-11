mod attribute;

use proc_macro::TokenStream;
use quote::{ToTokens, quote};

#[proc_macro_derive(AttributeGlobal)]
pub fn attribute_global(input: TokenStream) -> TokenStream {
    match attribute::derive(input.into(), quote!(wgame::shader)) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}

#[proc_macro_derive(Attribute)]
pub fn attribute(input: TokenStream) -> TokenStream {
    match attribute::derive(input.into(), quote!(wgame_shader)) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}
