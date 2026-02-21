//! Procedural macros for wgame application entry points.
//!
//! Provides `#[app]` and `#[window]` macros for creating application and window entry points.
//!
//! # Examples
//!
//! Single window application:
//!
//! ```rust
//! #[window(width = 800, height = 600)]
//! fn main() {
//!     // Window logic
//! }
//! ```
//!
//! Just application, windows need to be created manually:
//!
//! ```rust
//! use wgame_macros::{app, window};
//!
//! #[app]
//! fn main() {
//!     // Application logic
//! }
//! ```

mod entry_point;

use proc_macro::TokenStream;
use quote::ToTokens;

/// Marks a function as the main entry point for a wgame application.
#[proc_macro_attribute]
pub fn app(attr: TokenStream, item: TokenStream) -> TokenStream {
    match entry_point::impl_app(attr.into(), item.into()) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}

/// Marks a function as the main entry point for a wgame window.
#[proc_macro_attribute]
pub fn window(attr: TokenStream, item: TokenStream) -> TokenStream {
    match entry_point::impl_window(attr.into(), item.into()) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}
