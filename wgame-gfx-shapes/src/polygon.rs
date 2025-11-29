use std::fmt::{self, Debug};

use glam::{Affine3A, Mat3, Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use wgame_gfx::{modifiers::Transformed, types::Position};
use wgame_shader::Attribute;
use wgpu::util::DeviceExt;

use crate::{
    Shape, ShapeExt, ShapesLibrary, ShapesState,
    pipeline::create_pipeline,
    resource::VertexBuffers,
    shader::VertexData,
    shape::{Element, ShapeContext, Visitor},
};

#[derive(Clone)]
pub struct PolygonLibrary {
    pub triangle_vertices: wgpu::Buffer,
    pub quad_vertices: wgpu::Buffer,
    pub quad_indices: wgpu::Buffer,
    pub hexagon_vertices: wgpu::Buffer,
    pub hexagon_indices: wgpu::Buffer,
    pub pipeline: wgpu::RenderPipeline,
}

impl PolygonLibrary {
    pub fn new(state: &ShapesState) -> Self {
        let triangle_vertices =
            state
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("triangle_vertices"),
                    contents: &[
                        VertexData::new(Vec4::new(1.0, 0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0)),
                        VertexData::new(Vec4::new(0.0, 1.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 1.0)),
                        VertexData::new(Vec4::new(0.0, 0.0, 1.0, 1.0), Vec3::new(0.0, 1.0, 1.0)),
                    ]
                    .to_bytes(),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let quad_vertices = state
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("quad_vertices"),
                contents: &[
                    VertexData::new(Vec4::new(-1.0, -1.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 1.0)),
                    VertexData::new(Vec4::new(1.0, -1.0, 0.0, 1.0), Vec3::new(1.0, 1.0, 1.0)),
                    VertexData::new(Vec4::new(-1.0, 1.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0)),
                    VertexData::new(Vec4::new(1.0, 1.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 1.0)),
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
                    label: Some("hexagon_vertices"),
                    contents: &[
                        VertexData::new(Vec4::new(0.0, -1.0, 0.0, 1.0), Vec3::new(0.5, 1.0, 1.0)),
                        VertexData::new(
                            Vec4::new(sqrt_3_2, -0.5, 0.0, 1.0),
                            Vec3::new(0.5 + 0.5 * sqrt_3_2, 0.75, 1.0),
                        ),
                        VertexData::new(
                            Vec4::new(sqrt_3_2, 0.5, 0.0, 1.0),
                            Vec3::new(0.5 + 0.5 * sqrt_3_2, 0.25, 1.0),
                        ),
                        VertexData::new(Vec4::new(0.0, 1.0, 0.0, 1.0), Vec3::new(0.5, 0.0, 1.0)),
                        VertexData::new(
                            Vec4::new(-sqrt_3_2, 0.5, 0.0, 1.0),
                            Vec3::new(0.5 - 0.5 * sqrt_3_2, 0.25, 1.0),
                        ),
                        VertexData::new(
                            Vec4::new(-sqrt_3_2, -0.5, 0.0, 1.0),
                            Vec3::new(0.5 - 0.5 * sqrt_3_2, 0.75, 1.0),
                        ),
                    ]
                    .to_bytes(),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let hexagon_indices =
            state
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("hexagon_indices"),
                    contents: bytemuck::cast_slice::<u32, _>(&[0, 1, 2, 2, 3, 4, 4, 5, 0, 0, 2, 4]),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let pipeline =
            create_pipeline(state, &Default::default()).expect("Failed to create polygon pipeline");

        Self {
            triangle_vertices,
            quad_vertices,
            quad_indices,
            hexagon_vertices,
            hexagon_indices,
            pipeline,
        }
    }
}

#[derive(Clone)]
pub struct Polygon<const N: u32> {
    library: ShapesLibrary,
    vertices: wgpu::Buffer,
    indices: Option<wgpu::Buffer>,
    pipeline: wgpu::RenderPipeline,
}

impl<const N: u32> Element for Polygon<N> {
    type Attribute = ();

    fn state(&self) -> &ShapesState {
        &self.library.state
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

impl<const N: u32> Shape for Polygon<N> {
    fn library(&self) -> &ShapesLibrary {
        &self.library
    }
    fn visit<V: Visitor>(&self, ctx: ShapeContext, visitor: &mut V) {
        visitor.apply(ctx, self);
    }
}

pub type Triangle = Polygon<3>;
pub type Quad = Polygon<4>;
pub type Hexagon = Polygon<6>;

impl ShapesLibrary {
    pub fn triangle(
        &self,
        a: impl Position,
        b: impl Position,
        c: impl Position,
    ) -> Transformed<Polygon<3>> {
        Polygon {
            library: self.clone(),
            vertices: self.polygon.triangle_vertices.clone(),
            indices: None,
            pipeline: self.polygon.pipeline.clone(),
        }
        .transform(Mat4::from_mat3(Mat3::from_cols(
            a.to_xyzw().xyz(),
            b.to_xyzw().xyz(),
            c.to_xyzw().xyz(),
        )))
    }

    pub fn unit_quad(&self) -> Polygon<4> {
        Polygon {
            library: self.clone(),
            vertices: self.polygon.quad_vertices.clone(),
            indices: Some(self.polygon.quad_indices.clone()),
            pipeline: self.polygon.pipeline.clone(),
        }
    }

    pub fn rectangle(&self, (min, max): (Vec2, Vec2)) -> Transformed<Polygon<4>> {
        let center = 0.5 * (min + max);
        let half_size = 0.5 * (max - min);
        let affine = Affine3A::from_mat3_translation(
            Mat3::from_diagonal(Vec3::from((half_size, 1.0))),
            Vec3::from((center, 0.0)),
        );
        self.unit_quad().transform(affine)
    }

    pub fn unit_hexagon(&self) -> Polygon<6> {
        Polygon {
            library: self.clone(),
            vertices: self.polygon.hexagon_vertices.clone(),
            indices: Some(self.polygon.hexagon_indices.clone()),
            pipeline: self.polygon.pipeline.clone(),
        }
    }
}

impl<const N: u32> Debug for Polygon<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Polygon<{N}>")
    }
}
