#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

//mod circle;
mod pipeline;
mod polygon;
mod primitive;
mod shader;
mod shape;
mod textured;

use anyhow::Result;

use wgame_gfx::State;

use crate::{
    //circle::CircleRenderer,
    polygon::PolygonRenderer,
};

pub use self::{
    polygon::Polygon,
    shape::{Shape, ShapeExt},
    textured::Textured,
};

/// 2D graphics library
pub struct Library<'a> {
    state: State<'a>,
    polygon: PolygonRenderer,
    //circle: CircleRenderer,
}

impl<'a> Library<'a> {
    pub fn new(state: &State<'a>) -> Result<Self> {
        Ok(Self {
            state: state.clone(),
            polygon: PolygonRenderer::new(state)?,
            //circle: CircleRenderer::new(state)?,
        })
    }
}
