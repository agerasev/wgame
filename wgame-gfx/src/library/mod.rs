mod circle;
mod geometry;
mod pipeline;
mod polygon;
mod texture;

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec4};

use crate::{
    SharedState,
    library::{circle::CircleRenderer, polygon::PolygonRenderer},
};

pub use self::{
    geometry::{Geometry, GeometryExt},
    polygon::Polygon,
    texture::Texture,
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
    state: SharedState<'a>,
    polygon: PolygonRenderer,
    circle: CircleRenderer,
}

impl<'a> Library<'a> {
    pub fn new(state: &SharedState<'a>) -> Result<Self> {
        Ok(Self {
            state: state.clone(),
            polygon: PolygonRenderer::new(state)?,
            circle: CircleRenderer::new(state)?,
        })
    }
}
