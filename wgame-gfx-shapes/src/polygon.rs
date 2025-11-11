use glam::{Affine2, Affine3A, Mat2, Mat3, Vec2, Vec3, Vec4};
use wgame_gfx::types::Position;
use wgame_shader::Attribute;
use wgpu::util::DeviceExt;

use crate::{
    Shape, ShapeExt, ShapesLibrary, ShapesState, pipeline::create_pipeline, primitive::VertexData,
    renderer::VertexBuffers,
};

#[derive(Clone)]
pub struct PolygonLibrary {
    pub quad_vertices: wgpu::Buffer,
    pub quad_indices: wgpu::Buffer,
    pub hexagon_vertices: wgpu::Buffer,
    pub hexagon_indices: wgpu::Buffer,
    pub pipeline: wgpu::RenderPipeline,
}

impl PolygonLibrary {
    pub fn new(state: &ShapesState) -> Self {
        let quad_vertices = state
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("quad_vertices"),
                contents: &[
                    VertexData::new(Vec4::new(-1.0, -1.0, 0.0, 1.0), Vec2::new(0.0, 0.0)),
                    VertexData::new(Vec4::new(1.0, -1.0, 0.0, 1.0), Vec2::new(1.0, 0.0)),
                    VertexData::new(Vec4::new(-1.0, 1.0, 0.0, 1.0), Vec2::new(0.0, 1.0)),
                    VertexData::new(Vec4::new(1.0, 1.0, 0.0, 1.0), Vec2::new(1.0, 1.0)),
                ]
                .to_bytes(),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let quad_indices = state
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("quad_indices"),
                contents: bytemuck::cast_slice::<u32, _>(&[0, 1, 2, 2, 1, 3]),
                usage: wgpu::BufferUsages::INDEX,
            });

        let sqrt_3_2 = 3.0f32.sqrt() / 2.0;
        let hexagon_vertices =
            state
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("quad_vertices"),
                    contents: &[
                        VertexData::new(Vec4::new(0.0, -1.0, 0.0, 1.0), Vec2::new(0.5, 0.0)),
                        VertexData::new(
                            Vec4::new(sqrt_3_2, -0.5, 0.0, 1.0),
                            Vec2::new(0.5 + 0.5 * sqrt_3_2, 0.25),
                        ),
                        VertexData::new(
                            Vec4::new(sqrt_3_2, 0.5, 0.0, 1.0),
                            Vec2::new(0.5 + 0.5 * sqrt_3_2, 0.75),
                        ),
                        VertexData::new(Vec4::new(0.0, 1.0, 0.0, 1.0), Vec2::new(0.5, 1.0)),
                        VertexData::new(
                            Vec4::new(-sqrt_3_2, 0.5, 0.0, 1.0),
                            Vec2::new(0.5 - 0.5 * sqrt_3_2, 0.75),
                        ),
                        VertexData::new(
                            Vec4::new(-sqrt_3_2, -0.5, 0.0, 1.0),
                            Vec2::new(0.5 - 0.5 * sqrt_3_2, 0.25),
                        ),
                    ]
                    .to_bytes(),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let hexagon_indices =
            state
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("quad_indices"),
                    contents: bytemuck::cast_slice::<u32, _>(&[0, 1, 2, 2, 3, 4, 4, 5, 0, 0, 2, 4]),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let pipeline =
            create_pipeline(state, &Default::default()).expect("Failed to create polygon pipeline");

        Self {
            quad_vertices,
            quad_indices,
            hexagon_vertices,
            hexagon_indices,
            pipeline,
        }
    }
}

pub struct Polygon<const N: u32> {
    state: ShapesLibrary,
    vertices: wgpu::Buffer,
    indices: Option<wgpu::Buffer>,
    pipeline: wgpu::RenderPipeline,
}

impl<const N: u32> Shape for Polygon<N> {
    type Attribute = ();

    fn state(&self) -> &ShapesLibrary {
        &self.state
    }

    fn vertices(&self) -> VertexBuffers {
        VertexBuffers {
            count: 3 * (N - 2),
            vertex_buffer: self.vertices.clone(),
            index_buffer: self.indices.clone(),
        }
    }

    fn attribute(&self) -> Self::Attribute {}

    fn pipeline(&self) -> wgpu::RenderPipeline {
        self.pipeline.clone()
    }
}

impl ShapesLibrary {
    pub fn triangle(&self, a: impl Position, b: impl Position, c: impl Position) -> Polygon<3> {
        let vertices = self
            .state
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("triangle_vertices"),
                contents: &[
                    VertexData::new(a.to_xyzw(), Vec2::new(0.0, 0.0)),
                    VertexData::new(b.to_xyzw(), Vec2::new(1.0, 0.0)),
                    VertexData::new(c.to_xyzw(), Vec2::new(0.0, 1.0)),
                ]
                .to_bytes(),
                usage: wgpu::BufferUsages::VERTEX,
            });
        Polygon {
            state: self.clone(),
            vertices,
            indices: None,
            pipeline: self.polygon.pipeline.clone(),
        }
    }

    pub fn unit_quad(&self) -> Polygon<4> {
        Polygon {
            state: self.clone(),
            vertices: self.polygon.quad_vertices.clone(),
            indices: Some(self.polygon.quad_indices.clone()),
            pipeline: self.polygon.pipeline.clone(),
        }
    }

    pub fn quad(&self, a: Vec2, b: Vec2) -> impl Shape {
        let center = 0.5 * (a + b);
        let half_size = 0.5 * (b - a);
        let affine = Affine3A::from_mat3_translation(
            Mat3::from_diagonal(Vec3::from((half_size, 1.0))),
            Vec3::from((center, 0.0)),
        );
        self.unit_quad().transform(affine)
    }

    pub fn unit_hexagon(&self) -> Polygon<6> {
        Polygon {
            state: self.clone(),
            vertices: self.polygon.hexagon_vertices.clone(),
            indices: Some(self.polygon.hexagon_indices.clone()),
            pipeline: self.polygon.pipeline.clone(),
        }
    }

    pub fn hexagon(&self, center: Vec2, edge_size: f32) -> impl Shape {
        self.unit_hexagon()
            .transform(Affine2::from_mat2_translation(
                Mat2::from_diagonal(Vec2::new(edge_size, edge_size)),
                center,
            ))
    }
}
