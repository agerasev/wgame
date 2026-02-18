//! GPU-accelerated 2D shape rendering.
//!
//! Provides circle and polygon primitives with fill, stroke, and texture support.

#![forbid(unsafe_code)]

mod circle;
pub mod geometry;
mod pipeline;
mod polygon;
mod render;
pub mod shader;
mod shape;

use core::ops::Deref;
use wgame_gfx::{
    Graphics,
    types::{Color, color},
};
use wgame_gfx_texture::{Texture, TexturingLibrary, TexturingState};
use wgame_image::Image;

use crate::{circle::CircleLibrary, polygon::PolygonLibrary};

pub use self::{
    circle::{Circle, CircleFill, CircleStroke},
    geometry::Mesh,
    polygon::{Polygon, PolygonFill},
    shape::{Shape, Textured},
};

/// Commonly used shape traits.
pub mod prelude {
    pub use crate::shape::{Shape, ShapeFill, ShapeStroke, Textured};
}

/// Shared state for shape rendering.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ShapesState {
    texture: TexturingState,
}

impl ShapesState {
    /// Returns a reference to the texture state.
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

/// 2D graphics library for rendering shapes.
#[derive(Clone)]
pub struct ShapesLibrary {
    state: ShapesState,
    polygon: PolygonLibrary,
    circle: CircleLibrary,
    white_texture: Texture,
}

impl ShapesLibrary {
    /// Creates a new shapes library.
    pub fn new(state: &Graphics, texture: &TexturingLibrary) -> Self {
        assert_eq!(state, &**texture.state());
        let state = ShapesState {
            texture: texture.state().clone(),
        };
        Self {
            polygon: PolygonLibrary::new(&state),
            circle: CircleLibrary::new(&state),
            white_texture: texture.texture(
                &Image::with_color((1, 1), color::WHITE.to_rgba_f16()),
                Default::default(),
            ),
            state,
        }
    }

    /// Returns a reference to the shapes state.
    pub fn state(&self) -> &ShapesState {
        &self.state
    }
}
