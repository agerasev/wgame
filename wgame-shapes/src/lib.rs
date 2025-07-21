#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

pub mod attributes;
pub mod binding;
pub mod bytes;
mod circle;
mod pipeline;
mod polygon;
pub mod primitive;
mod renderer;
mod shader;
mod shape;
mod textured;

pub use self::{
    polygon::Polygon,
    shape::{Shape, ShapeExt},
    textured::{Textured, gradient, gradient2},
};

use wgame_gfx::{
    Graphics, Texture,
    registry::{RegistryInit, RegistryKey},
    types::{Color, color},
};

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct WhiteTextureKey;
impl RegistryKey for WhiteTextureKey {
    type Value = Texture;
}
impl RegistryInit for WhiteTextureKey {
    fn create_value(&self, state: &Graphics) -> Self::Value {
        Texture::with_data(state, (1, 1), &[color::WHITE.to_rgba()])
    }
}

pub trait GraphicsShapes {}
