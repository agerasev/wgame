//! Macros for deriving shader attribute implementations.
//!
//! This crate provides procedural macros that derive the [`wgame_shader::Attribute`]
//! trait for structs, allowing them to be used as shader input/output structures.
//!
//! # Derivable Types
//!
//! Only `struct` types can derive the `Attribute` trait. Enums and unions are not
//! supported.
//!
//! # Attribute Global
//!
//! The `AttributeGlobal` derive macro is used for global shader attributes that
//! don't need prefixing. It generates bindings using the `wgame::shader` module path.
//!
//! # Attribute
//!
//! The `Attribute` derive macro is used for regular shader attributes. It generates
//! bindings with the `wgame_shader` module path and supports field prefixing.
//!
//! # Examples
//!
//! Basic struct with typed fields:
//! ```
//! # use wgame_shader_macros::Attribute;
//! #[derive(Attribute)]
//! struct Vertex {
//!     position: [f32; 2],
//!     color: [f32; 4],
//! }
//! ```
//!
//! Using AttributeGlobal for global uniforms:
//! ```
//! # use wgame_shader_macros::AttributeGlobal;
//! #[derive(AttributeGlobal)]
//! struct GlobalUniforms {
//!     projection: [[f32; 4]; 4],
//!     view: [[f32; 4]; 4],
//! }
//! ```
//!
//! # Generated Implementation
//!
//! For a struct with fields, this generates:
//! - A [`wgame_shader::Attribute::bindings`] implementation that creates a
//!   [`wgame_shader::BindingList`] with prefixed binding names
//! - A [`wgame_shader::Attribute::SIZE`] constant indicating the total size
//! - A [`wgame_shader::Attribute::store`] method that serializes the struct
//!   into a byte sink

mod attribute;

use proc_macro::TokenStream;
use quote::{ToTokens, quote};

/// Derives the [`wgame_shader::Attribute`] trait for a struct with global (unprefixed) bindings.
///
/// This macro is intended for global shader uniforms that don't require field prefixing.
/// It uses the `wgame::shader` module path for the generated implementation.
///
/// # Examples
///
/// ```
/// # use wgame_shader_macros::AttributeGlobal;
/// #[derive(AttributeGlobal)]
/// struct CameraUniforms {
///     view_projection: [[f32; 4]; 4],
/// }
/// ```
#[proc_macro_derive(AttributeGlobal)]
pub fn attribute_global(input: TokenStream) -> TokenStream {
    match attribute::derive(input.into(), quote!(wgame::shader)) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}

/// Derives the [`wgame_shader::Attribute`] trait for a struct with prefixed bindings.
///
/// This macro is used for regular shader attributes where field names should be
/// prefixed when generating bindings. It uses the `wgame_shader` module path
/// for the generated implementation.
///
/// # Examples
///
/// ```
/// # use wgame_shader_macros::Attribute;
/// #[derive(Attribute)]
/// struct VertexAttributes {
///     position: [f32; 3],
///     normal: [f32; 3],
///     tex_coords: [f32; 2],
/// }
/// ```
#[proc_macro_derive(Attribute)]
pub fn attribute(input: TokenStream) -> TokenStream {
    match attribute::derive(input.into(), quote!(wgame_shader)) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}
