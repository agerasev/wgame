//! Procedural macros for deriving shader attribute implementations.
//!
//! Provides `#[derive(Attribute)]` and `#[derive(AttributeGlobal)]` for structs.
//!
//! # Examples
//!
//! ```rust
//! use wgame_shader_macros::{Attribute, AttributeGlobal};
//!
//! #[derive(Attribute)]
//! struct Vertex {
//!     position: [f32; 3],
//!     normal: [f32; 3],
//! }
//!
//! #[derive(AttributeGlobal)]
//! struct GlobalUniforms {
//!     projection: glam::Mat4,
//!     view: glam::Mat4,
//! }
//! ```

mod attribute;

use proc_macro::TokenStream;
use quote::{ToTokens, quote};

/// Derives the `Attribute` trait for a struct.
#[proc_macro_derive(AttributeGlobal)]
pub fn attribute_global(input: TokenStream) -> TokenStream {
    match attribute::derive(input.into(), quote!(wgame::shader)) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}

/// Derives the `Attribute` trait for a struct (for internal usage).
#[proc_macro_derive(Attribute)]
pub fn attribute(input: TokenStream) -> TokenStream {
    match attribute::derive(input.into(), quote!(wgame_shader)) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}
