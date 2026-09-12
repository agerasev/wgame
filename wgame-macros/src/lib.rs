//! Procedural macros for wgame application entry points.
//!
//! Provides `#[app]` and `#[window]` macros for creating application and window entry points.
//!
//! # Examples
//!
//! Single window application:
//!
//! ```no_run
//! #[wgame::app]
//! async fn main() {
//!     // Start tasks or create windows here.
//! }
//! ```
//!
//! ```no_run
//! #[wgame::window(size = (800, 600))]
//! async fn main(mut window: wgame::Window<'_>) -> wgame::Result<()> {
//!     while let Some(_frame) = window.next_frame().await? {}
//!     Ok(())
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
