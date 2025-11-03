#![forbid(unsafe_code)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod atlas;
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

pub mod prelude {
    pub use crate::traits::*;
}
