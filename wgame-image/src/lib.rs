//! Image processing utilities for wgame.
//!
//! This crate provides types and traits for working with 2D images, including:
//! - [`Image`] - A simple 2D image container
//! - [`Atlas`] - A texture atlas for efficiently packing multiple images
//! - [`Pixel`] - Trait for pixel types
//! - [`ImageRead`], [`ImageWrite`] - Traits for reading/writing image data
//!
//! # Overview
//!
//! The crate provides a flexible image system with support for:
//! - Multiple pixel formats (u8, f16, RGBA)
//! - Image slicing and manipulation
//! - Texture atlasing for efficient GPU uploads
//! - Image encoding/decoding (PNG support)
//!
//! # Pixel Types
//!
//! The [`Pixel`] trait is implemented for:
//! - [`u8`] - Single channel grayscale
//! - [`Rgba<u8>`] - 8-bit RGBA color
//! - [`f16`] - Single channel half-float
//! - [`Rgba<f16>`] - 16-bit RGBA color
//!
//! # Image Operations
//!
//! Images support various operations:
//! - [`Image::new`] / [`Image::with_color`] - Create images
//! - [`ImageReadExt::slice`] - Get a sub-region of an image
//! - [`ImageReadExt::get`] - Get a single pixel
//! - [`ImageReadExt::pixels`] - Iterate over all pixels
//! - [`ImageWriteMut::copy_from`] - Copy from another image
//! - [`ImageWriteMut::fill`] - Fill with a color
//!
//! # Texture Atlasing
//!
//! The [`Atlas`] type provides an efficient way to pack multiple images into
//! a single texture. It uses the [`guillotiere`] crate for the underlying
//! allocation algorithm.
//!
//! ## Example
//!
//! ```
//! use wgame_image::{Atlas, Image, Rgba, f16};
//!
//! let mut atlas = Atlas::<Rgba<f16>>::default();
//!
//! // Allocate space for an image
//! let image = atlas.allocate((64, 64));
//!
//! // Update the allocated region
//! image.update(|slice| {
//!     // Fill with white color
//!     slice.fill(Rgba::new(1.0, 1.0, 1.0, 1.0));
//! });
//! ```
//!
//! # Encoding/Decoding
//!
//! When the `image` feature is enabled, the crate provides:
//! - [`Encoding`] - Supported image formats
//! - [`Image::decode`] - Decode bytes to an image
//! - [`ImageSlice::encode`] - Encode an image slice to bytes
//!
//! Note: The crate converts images to sRGB space during decoding/encoding.

#![forbid(unsafe_code)]

pub mod atlas;
#[cfg(feature = "image")]
mod endec;
mod image;
mod pixel;
mod slice;
#[cfg(test)]
mod tests;
mod traits;

pub use crate::{
    atlas::{Atlas, AtlasImage},
    image::Image,
    pixel::Pixel,
    slice::{ImageSlice, ImageSliceMut},
    traits::*,
};
#[cfg(feature = "image")]
pub use endec::Encoding;

/// Re-export of commonly used traits for convenient importing.
pub mod prelude {
    pub use crate::traits::*;
}
