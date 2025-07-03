use std::rc::Rc;

use crate::{State, object::Vertices};

use super::Geometry;

pub struct Polygon<'a> {
    pub(crate) vertex_count: u32,
    pub(crate) state: Rc<State<'a>>,
    pub(crate) vertices: wgpu::Buffer,
    pub(crate) indices: Option<wgpu::Buffer>,
    pub(crate) pipeline: wgpu::RenderPipeline,
}

impl<'a> Geometry<'a> for Polygon<'a> {
    fn state(&self) -> &Rc<State<'a>> {
        &self.state
    }

    fn vertices(&self) -> Vertices {
        Vertices {
            count: 3 * (self.vertex_count - 2),
            vertex_buffer: self.vertices.clone(),
            index_buffer: self.indices.clone(),
        }
    }

    fn pipeline(&self) -> wgpu::RenderPipeline {
        self.pipeline.clone()
    }
}
