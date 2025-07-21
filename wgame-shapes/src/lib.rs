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

use alloc::rc::Rc;

use anyhow::Result;

use wgame_gfx::{
    Graphics, Texture,
    types::{Color, color},
};

use crate::{circle::CircleRenderer, polygon::PolygonRenderer};

pub use self::{
    polygon::Polygon,
    shape::{Shape, ShapeExt},
    textured::{Textured, gradient, gradient2},
};

struct InnerLibrary {
    state: Graphics,
    polygon: PolygonRenderer,
    circle: CircleRenderer,
    white_texture: Texture,
}

/// 2D graphics library
#[derive(Clone)]
pub struct Library(Rc<InnerLibrary>);

impl Library {
    pub fn new(state: &Graphics) -> Result<Self> {
        Ok(Self(Rc::new(InnerLibrary {
            state: state.clone(),
            polygon: PolygonRenderer::new(state)?,
            circle: CircleRenderer::new(state)?,
            white_texture: { Texture::with_data(state, (1, 1), &[color::WHITE.to_rgba()]) },
        })))
    }
}
