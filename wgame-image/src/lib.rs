//! 2D image processing and texture atlas utilities.
//!
//! Provides image containers, pixel formats, slicing operations, and texture atlasing.

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

/// Commonly used traits.
pub mod prelude {
    #[doc(no_inline)]
    pub use crate::traits::*;
}
