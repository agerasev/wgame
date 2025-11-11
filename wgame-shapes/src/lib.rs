#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

pub mod attributes;
pub mod binding;
mod circle;
mod instance;
mod pipeline;
mod polygon;
pub mod primitive;
mod renderer;
mod shader;
mod shape;

use core::ops::Deref;
use wgame_gfx::{
    Graphics,
    types::{Color, color},
};
use wgame_image::Image;
use wgame_texture::{Texture, TextureLibrary, TextureState};

use crate::{circle::CircleLibrary, polygon::PolygonLibrary};

pub use self::{
    instance::Textured,
    polygon::Polygon,
    shape::{Shape, ShapeExt},
};

/// Library shared state
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ShapesState {
    texture: TextureState,
}

impl ShapesState {
    pub fn texture(&self) -> &TextureState {
        &self.texture
    }
}

impl Deref for ShapesState {
    type Target = Graphics;
    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

/// 2D graphics library
#[derive(Clone)]
pub struct ShapesLibrary {
    state: ShapesState,
    polygon: PolygonLibrary,
    circle: CircleLibrary,
    white_texture: Texture,
}

impl Deref for ShapesLibrary {
    type Target = ShapesState;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl ShapesLibrary {
    pub fn new(state: &Graphics, texture: &TextureLibrary) -> Self {
        assert_eq!(state, &**texture.state());
        let state = ShapesState {
            texture: texture.state().clone(),
        };
        Self {
            polygon: PolygonLibrary::new(&state),
            circle: CircleLibrary::new(&state),
            white_texture: texture.texture(&Image::with_color((1, 1), color::WHITE.to_rgba())),
            state,
        }
    }
}
