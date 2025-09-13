#![forbid(unsafe_code)]

mod image;
mod pixel;
#[cfg(test)]
mod tests;

pub use crate::{
    image::{Image, ImageSlice, ImageSliceMut},
    pixel::Pixel,
};

pub mod prelude {
    pub use crate::image::{ImageLike, ImageLikeExt, ImageLikeMut, ImageLikeMutExt};
}
