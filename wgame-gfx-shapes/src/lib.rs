#![forbid(unsafe_code)]

mod circle;
mod pipeline;
mod polygon;
mod resource;
pub mod shader;
mod shape;
mod textured;

use core::ops::Deref;
use wgame_gfx::{
    Graphics,
    types::{Color, color},
};
use wgame_gfx_texture::{Texture, TexturingLibrary, TexturingState};
use wgame_image::Image;

use crate::{circle::CircleLibrary, polygon::PolygonLibrary};

pub use self::{
    polygon::{Hexagon, Polygon, Quad, Triangle},
    shape::{Shape, ShapeExt},
    textured::Textured,
};

/// Library shared state
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ShapesState {
    texture: TexturingState,
}

impl ShapesState {
    pub fn texture(&self) -> &TexturingState {
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
    pub fn new(state: &Graphics, texture: &TexturingLibrary) -> Self {
        assert_eq!(state, &**texture.state());
        let state = ShapesState {
            texture: texture.state().clone(),
        };
        Self {
            polygon: PolygonLibrary::new(&state),
            circle: CircleLibrary::new(&state),
            white_texture: texture.texture(
                &Image::with_color((1, 1), color::WHITE.to_rgba()),
                Default::default(),
            ),
            state,
        }
    }
}
