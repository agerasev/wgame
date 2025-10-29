#![forbid(unsafe_code)]

mod image;
mod pixel;
#[cfg(test)]
mod tests;
mod traits;

pub use crate::{
    image::{Image, ImageSlice, ImageSliceMut},
    pixel::Pixel,
    traits::*,
};

pub mod prelude {
    pub use crate::traits::*;
}
