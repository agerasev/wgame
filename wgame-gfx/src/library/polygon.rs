use glam::{Affine3A, Mat3, Mat4, Vec2, Vec3};

use crate::{SharedState, Transformed, object::Vertices};

use super::{Geometry, GeometryExt};

pub struct Polygon<'a, const N: u32> {
    pub(crate) state: SharedState<'a>,
    pub(crate) vertices: wgpu::Buffer,
    pub(crate) indices: Option<wgpu::Buffer>,
    pub(crate) pipeline: wgpu::RenderPipeline,
}

impl<'a, const N: u32> Geometry<'a> for Polygon<'a, N> {
    fn state(&self) -> &SharedState<'a> {
        &self.state
    }

    fn vertices(&self) -> Vertices {
        Vertices {
            count: 3 * (N - 2),
            vertex_buffer: self.vertices.clone(),
            index_buffer: self.indices.clone(),
        }
    }

    fn transformation(&self) -> Mat4 {
        Mat4::IDENTITY
    }

    fn pipeline(&self) -> wgpu::RenderPipeline {
        self.pipeline.clone()
    }
}

impl Polygon<'_, 3> {
    pub fn transform_vertices(self, vertices: [Vec3; 3]) -> Transformed<Self> {
        let offset = vertices[0];
        let affine = Affine3A::from_mat3_translation(
            Mat3::from_cols(vertices[1] - offset, vertices[2] - offset, Vec3::Z),
            offset,
        );
        self.transform(affine.into())
    }
}

impl Polygon<'_, 4> {
    pub fn transform_to_rect(self, top_left: Vec2, bottom_right: Vec2) -> Transformed<Self> {
        let size = bottom_right - top_left;
        let affine = Affine3A::from_mat3_translation(
            Mat3::from_cols(
                Vec3::new(size.x, 0.0, 0.0),
                Vec3::new(0.0, size.y, 0.0),
                Vec3::Z,
            ),
            Vec3::from((top_left, 0.0)),
        );
        self.transform(affine.into())
    }
}
