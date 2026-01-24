use std::fmt::{self, Debug};

use glam::{Affine3A, Mat3, Vec2, Vec3, Vec4};
use wgame_gfx::{
    modifiers::Transformable,
    types::{Position, Transform},
};
use wgame_shader::Attribute;
use wgpu::util::DeviceExt;

use crate::{
    Shape, ShapesLibrary, ShapesState,
    pipeline::create_pipeline,
    render::VertexBuffers,
    shader::VertexData,
    shape::{Element, ElementVisitor},
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
                        VertexData::new(Vec4::new(1.0, 0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 1.0)),
                        VertexData::new(Vec4::new(0.0, 1.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 1.0)),
                        VertexData::new(Vec4::new(0.0, 0.0, 1.0, 1.0), Vec3::new(0.0, 0.0, 1.0)),
                    ]
                    .to_bytes(),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let quad_vertices = state
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("quad_vertices"),
                contents: &[
                    VertexData::new(Vec4::new(-1.0, -1.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0)),
                    VertexData::new(Vec4::new(1.0, -1.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 1.0)),
                    VertexData::new(Vec4::new(-1.0, 1.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 1.0)),
                    VertexData::new(Vec4::new(1.0, 1.0, 0.0, 1.0), Vec3::new(1.0, 1.0, 1.0)),
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
                        VertexData::new(Vec4::new(0.0, -1.0, 0.0, 1.0), Vec3::new(0.5, 0.0, 1.0)),
                        VertexData::new(
                            Vec4::new(sqrt_3_2, -0.5, 0.0, 1.0),
                            Vec3::new(0.5 + 0.5 * sqrt_3_2, 0.25, 1.0),
                        ),
                        VertexData::new(
                            Vec4::new(sqrt_3_2, 0.5, 0.0, 1.0),
                            Vec3::new(0.5 + 0.5 * sqrt_3_2, 0.75, 1.0),
                        ),
                        VertexData::new(Vec4::new(0.0, 1.0, 0.0, 1.0), Vec3::new(0.5, 1.0, 1.0)),
                        VertexData::new(
                            Vec4::new(-sqrt_3_2, 0.5, 0.0, 1.0),
                            Vec3::new(0.5 - 0.5 * sqrt_3_2, 0.75, 1.0),
                        ),
                        VertexData::new(
                            Vec4::new(-sqrt_3_2, -0.5, 0.0, 1.0),
                            Vec3::new(0.5 - 0.5 * sqrt_3_2, 0.25, 1.0),
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

#[must_use]
#[derive(Clone)]
pub struct Polygon {
    library: ShapesLibrary,
    count: u32,
    vertices: wgpu::Buffer,
    indices: Option<wgpu::Buffer>,
    pipeline: wgpu::RenderPipeline,
    xform: Affine3A,
}

impl Element for Polygon {
    type Attribute = ();

    fn state(&self) -> &ShapesState {
        &self.library.state
    }

    fn vertices(&self) -> VertexBuffers {
        VertexBuffers {
            count: 3 * (self.count - 2),
            vertex_buffer: self.vertices.clone(),
            index_buffer: self.indices.clone(),
        }
    }

    fn attribute(&self) -> Self::Attribute {}

    fn pipeline(&self) -> wgpu::RenderPipeline {
        self.pipeline.clone()
    }

    fn xform(&self) -> Affine3A {
        self.xform
    }
}

impl Shape for Polygon {
    fn library(&self) -> &ShapesLibrary {
        &self.library
    }
    fn for_each_element<V: ElementVisitor>(&self, visitor: &mut V) {
        visitor.visit(self);
    }
}

impl Transformable for Polygon {
    fn transform<X: Transform>(&self, xform: X) -> Self {
        Self {
            xform: xform.to_affine3() * self.xform,
            ..self.clone()
        }
    }
}

impl ShapesLibrary {
    fn polygon(
        &self,
        count: u32,
        vertices: &wgpu::Buffer,
        indices: Option<&wgpu::Buffer>,
    ) -> Polygon {
        Polygon {
            library: self.clone(),
            count,
            vertices: vertices.clone(),
            indices: indices.cloned(),
            pipeline: self.polygon.pipeline.clone(),
            xform: Affine3A::IDENTITY,
        }
    }

    pub fn triangle(&self, a: impl Position, b: impl Position, c: impl Position) -> Polygon {
        self.polygon(3, &self.polygon.triangle_vertices, None)
            .transform(Mat3::from_cols(a.to_xyz(), b.to_xyz(), c.to_xyz()))
    }

    pub fn unit_quad(&self) -> Polygon {
        self.polygon(
            4,
            &self.polygon.quad_vertices,
            Some(&self.polygon.quad_indices),
        )
    }

    pub fn rectangle(&self, (min, max): (Vec2, Vec2)) -> Polygon {
        let center = 0.5 * (min + max);
        let half_size = 0.5 * (max - min);
        let affine = Affine3A::from_mat3_translation(
            Mat3::from_diagonal(Vec3::from((half_size, 1.0))),
            Vec3::from((center, 0.0)),
        );
        self.unit_quad().transform(affine)
    }

    pub fn unit_hexagon(&self) -> Polygon {
        self.polygon(
            6,
            &self.polygon.hexagon_vertices,
            Some(&self.polygon.hexagon_indices),
        )
    }
}

impl Debug for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Polygon<{}>", self.count)
    }
}
