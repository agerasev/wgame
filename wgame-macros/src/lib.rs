mod attributes;
mod main_;
mod store_bytes;

use proc_macro::TokenStream;
use quote::ToTokens;

#[proc_macro_attribute]
pub fn app(attr: TokenStream, item: TokenStream) -> TokenStream {
    match main_::impl_app(attr.into(), item.into()) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn window(attr: TokenStream, item: TokenStream) -> TokenStream {
    match main_::impl_window(attr.into(), item.into()) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}

#[proc_macro_derive(StoreBytes, attributes(bytes_mod))]
pub fn store_bytes(input: TokenStream) -> TokenStream {
    match store_bytes::derive(input.into()) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}

#[proc_macro_derive(Attributes, attributes(attributes_mod))]
pub fn attributes(input: TokenStream) -> TokenStream {
    match attributes::derive(input.into()) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}
