#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

mod circle;
mod pipeline;
mod polygon;
mod shader;
mod shape;

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec4};

use wgame_gfx::State;

use crate::{circle::CircleRenderer, polygon::PolygonRenderer};

pub use self::{
    polygon::Polygon,
    shape::{Shape, ShapeExt},
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    pos: [f32; 4],
    local_coord: [f32; 2],
}

impl Vertex {
    fn new(pos: Vec4, local_coord: Vec2) -> Self {
        Self {
            pos: pos.into(),
            local_coord: local_coord.into(),
        }
    }
}

/// 2D graphics library
pub struct Library<'a> {
    state: State<'a>,
    polygon: PolygonRenderer,
    circle: CircleRenderer,
}

impl<'a> Library<'a> {
    pub fn new(state: &State<'a>) -> Result<Self> {
        Ok(Self {
            state: state.clone(),
            polygon: PolygonRenderer::new(state)?,
            circle: CircleRenderer::new(state)?,
        })
    }
}
