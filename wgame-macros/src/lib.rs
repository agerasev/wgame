//! Procedural macros for application entry points.
//!
//! This crate provides macros to simplify the creation of wgame applications
//! by automatically generating the boilerplate code needed to run applications
//! and windows.
//!
//! # Macros
//!
//! - [`app`] - Marks a function as the main entry point for a wgame application
//! - [`window`] - Marks a function as the main entry point for a wgame window
//!
//! # Examples
//!
//! Creating an application with the `app` macro:
//! ```
//! # use wgame_macros::app;
//! #[app]
//! fn main() {
//!     // Application logic here
//! }
//! ```
//!
//! Creating a window with custom configuration using the `window` macro:
//! ```
//! # use wgame_macros::window;
//! #[window(width = 800, height = 600)]
//! fn main() {
//!     // Window logic here
//! }
//! ```

mod entry_point;

use proc_macro::TokenStream;
use quote::ToTokens;

/// Marks a function as the main entry point for a wgame application.
///
/// This macro transforms the annotated function into an application entry point
/// by generating the necessary boilerplate to initialize and run a wgame application.
/// The function should have the signature `fn main()`.
///
/// The macro generates:
/// - A renamed version of the original function (prefixed with `__wgame_main`)
/// - A call to `wgame::run_app!` which sets up the application runtime and
///   executes the main function
///
/// # Examples
///
/// ```
/// # use wgame_macros::app;
/// #[app]
/// fn main() {
///     println!("Hello, wgame!");
/// }
/// ```
///
/// This expands to something equivalent to:
/// ```ignore
/// fn __wgame_main() {
///     println!("Hello, wgame!");
/// }
///
/// wgame::run_app!(main, __wgame_main, {});
/// ```
#[proc_macro_attribute]
pub fn app(attr: TokenStream, item: TokenStream) -> TokenStream {
    match entry_point::impl_app(attr.into(), item.into()) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}

/// Marks a function as the main entry point for a wgame window.
///
/// This macro transforms the annotated function into a window entry point
/// by generating the necessary boilerplate to initialize and run a wgame window
/// with optional configuration. The function should have the signature `fn main()`.
///
/// The macro accepts configuration attributes in the form `key = value`:
/// - `width: u32` - Window width in pixels
/// - `height: u32` - Window height in pixels
/// - And other [`wgame::WindowConfig`] fields
///
/// # Examples
///
/// Basic window:
/// ```
/// # use wgame_macros::window;
/// #[window]
/// fn main() {
///     // Window logic here
/// }
/// ```
///
/// Window with custom dimensions:
/// ```
/// # use wgame_macros::window;
/// #[window(width = 1280, height = 720)]
/// fn main() {
///     // Window logic here
/// }
/// ```
///
/// This macro expands to a call to `wgame::run_window!` with the specified
/// configuration.
#[proc_macro_attribute]
pub fn window(attr: TokenStream, item: TokenStream) -> TokenStream {
    match entry_point::impl_window(attr.into(), item.into()) {
        Ok(expr) => expr.into_token_stream(),
        Err(err) => err.into_compile_error(),
    }
    .into()
}
