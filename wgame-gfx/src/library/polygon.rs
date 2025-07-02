use std::rc::Rc;

use crate::{State, object::Vertices};

use super::Geometry;

pub struct Polygon<'a, 'b> {
    pub(crate) vertex_count: u32,
    pub(crate) state: &'b Rc<State<'a>>,
    pub(crate) vertices: &'b wgpu::Buffer,
    pub(crate) indices: &'b wgpu::Buffer,
    pub(crate) pipeline: &'b wgpu::RenderPipeline,
}

impl<'a> Geometry<'a> for Polygon<'a, '_> {
    fn state(&self) -> &Rc<State<'a>> {
        self.state
    }

    fn vertices(&self) -> Vertices<'_> {
        Vertices {
            count: 3 * (self.vertex_count - 2),
            vertex_buffer: self.vertices,
            index_buffer: self.indices,
        }
    }

    fn pipeline(&self) -> &wgpu::RenderPipeline {
        self.pipeline
    }
}
