use anyhow::Result;
use glam::{Affine3A, Mat3, Mat4, Vec2, Vec3, Vec4};
use wgpu::util::DeviceExt;

use crate::{SharedState, library::pipeline::create_pipeline, object::Vertices, types::Position};

use super::{Geometry, GeometryExt, Library, Vertex};

pub struct PolygonRenderer {
    pub quad_vertices: wgpu::Buffer,
    pub quad_indices: wgpu::Buffer,
    pub pipeline: wgpu::RenderPipeline,
}

impl PolygonRenderer {
    pub fn new(state: &SharedState<'_>) -> Result<Self> {
        let quad_vertices = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("quad_vertices"),
                contents: bytemuck::cast_slice(&[
                    Vertex::new(Vec4::new(0.0, 0.0, 0.0, 1.0), Vec2::new(0.0, 0.0)),
                    Vertex::new(Vec4::new(1.0, 0.0, 0.0, 1.0), Vec2::new(1.0, 0.0)),
                    Vertex::new(Vec4::new(0.0, 1.0, 0.0, 1.0), Vec2::new(0.0, 1.0)),
                    Vertex::new(Vec4::new(1.0, 1.0, 0.0, 1.0), Vec2::new(1.0, 1.0)),
                ]),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let quad_indices = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("quad_indices"),
                contents: bytemuck::cast_slice::<u32, _>(&[0, 1, 2, 2, 1, 3]),
                usage: wgpu::BufferUsages::INDEX,
            });

        let pipeline = create_pipeline(state)?;

        Ok(Self {
            quad_vertices,
            quad_indices,
            pipeline,
        })
    }
}

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

impl<'a> Library<'a> {
    pub fn triangle(&self, a: impl Position, b: impl Position, c: impl Position) -> Polygon<'a, 3> {
        let vertices = self
            .state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("triangle_vertices"),
                contents: bytemuck::cast_slice(&[
                    Vertex::new(a.to_xyzw(), Vec2::new(0.0, 0.0)),
                    Vertex::new(b.to_xyzw(), Vec2::new(1.0, 0.0)),
                    Vertex::new(c.to_xyzw(), Vec2::new(0.0, 1.0)),
                ]),
                usage: wgpu::BufferUsages::VERTEX,
            });
        Polygon {
            state: self.state.clone(),
            vertices,
            indices: None,
            pipeline: self.polygon.pipeline.clone(),
        }
    }

    pub fn unit_quad(&self) -> Polygon<'a, 4> {
        Polygon {
            state: self.state.clone(),
            vertices: self.polygon.quad_vertices.clone(),
            indices: Some(self.polygon.quad_indices.clone()),
            pipeline: self.polygon.pipeline.clone(),
        }
    }

    pub fn quad(&self, a: Vec2, b: Vec2) -> impl Geometry<'a> {
        let min = a.min(b);
        let max = a.max(b);
        let size = max - min;
        let affine = Affine3A::from_mat3_translation(
            Mat3::from_cols(
                Vec3::new(size.x, 0.0, 0.0),
                Vec3::new(0.0, size.y, 0.0),
                Vec3::Z,
            ),
            Vec3::from((a, 0.0)),
        );
        self.unit_quad().transform(affine)
    }
}
