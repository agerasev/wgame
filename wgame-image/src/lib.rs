//! CPU images, slices, pixel types, and append-only atlas generations.
//!
//! [`Image`] owns pixels; [`ImageSlice`] and [`ImageSliceMut`] provide borrowed views.
//! [`Atlas`] shares storage between [`AtlasImage`] handles; see its allocation and
//! relocation contracts before integrating a GPU mirror.
//!
//! The default `png` feature enables PNG decoding and encoding. Pixel values are
//! passed through without automatic sRGB-to-linear conversion. [`euclid`] is
//! re-exported for rectangle and size arguments.

#![forbid(unsafe_code)]

pub mod atlas;
/// Geometry types used by image and atlas APIs.
pub use euclid;
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
