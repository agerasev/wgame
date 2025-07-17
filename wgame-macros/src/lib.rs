mod attributes;
mod main_;
mod store_bytes;

use proc_macro::TokenStream;
use quote::ToTokens;

#[proc_macro_attribute]
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    match main_::impl_(attr.into(), item.into()) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}

#[proc_macro_derive(StoreBytes)]
pub fn store_bytes(input: TokenStream) -> TokenStream {
    match store_bytes::derive(input.into()) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}

#[proc_macro_derive(Attributes)]
pub fn attributes(input: TokenStream) -> TokenStream {
    match attributes::derive(input.into()) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}
