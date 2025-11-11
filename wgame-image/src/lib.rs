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

pub mod prelude {
    pub use crate::traits::*;
}
