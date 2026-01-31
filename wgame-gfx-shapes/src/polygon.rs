use std::fmt::{self, Debug};

use glam::{Affine3A, Mat3, Vec2, Vec3, Vec4};
use wgame_gfx::{
    modifiers::Transformable,
    types::{Position, Transform},
};

use crate::{
    Mesh, Shape, ShapesLibrary, ShapesState,
    pipeline::create_pipeline,
    shader::Vertex,
    shape::{Element, ElementVisitor},
};

#[derive(Clone)]
pub struct PolygonLibrary {
    pub triangle: Mesh,
    pub quad: Mesh,
    pub hexagon: Mesh,
    pub pipeline: wgpu::RenderPipeline,
}

impl PolygonLibrary {
    pub fn new(state: &ShapesState) -> Self {
        let triangle = Mesh::from_arrays(
            state,
            &[
                Vertex::new(Vec4::new(1.0, 0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 1.0)),
                Vertex::new(Vec4::new(0.0, 1.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 1.0)),
                Vertex::new(Vec4::new(0.0, 0.0, 1.0, 1.0), Vec3::new(0.0, 0.0, 1.0)),
            ],
            None,
        );
        let quad = Mesh::from_arrays(
            state,
            &[
                Vertex::new(Vec4::new(-1.0, -1.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0)),
                Vertex::new(Vec4::new(1.0, -1.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 1.0)),
                Vertex::new(Vec4::new(-1.0, 1.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 1.0)),
                Vertex::new(Vec4::new(1.0, 1.0, 0.0, 1.0), Vec3::new(1.0, 1.0, 1.0)),
            ],
            Some(&[0, 1, 2, 2, 1, 3]),
        );

        let sqrt_3_2 = 3.0f32.sqrt() / 2.0;
        let hexagon = Mesh::from_arrays(
            state,
            &[
                Vertex::new(Vec4::new(0.0, -1.0, 0.0, 1.0), Vec3::new(0.5, 0.0, 1.0)),
                Vertex::new(
                    Vec4::new(sqrt_3_2, -0.5, 0.0, 1.0),
                    Vec3::new(0.5 + 0.5 * sqrt_3_2, 0.25, 1.0),
                ),
                Vertex::new(
                    Vec4::new(sqrt_3_2, 0.5, 0.0, 1.0),
                    Vec3::new(0.5 + 0.5 * sqrt_3_2, 0.75, 1.0),
                ),
                Vertex::new(Vec4::new(0.0, 1.0, 0.0, 1.0), Vec3::new(0.5, 1.0, 1.0)),
                Vertex::new(
                    Vec4::new(-sqrt_3_2, 0.5, 0.0, 1.0),
                    Vec3::new(0.5 - 0.5 * sqrt_3_2, 0.75, 1.0),
                ),
                Vertex::new(
                    Vec4::new(-sqrt_3_2, -0.5, 0.0, 1.0),
                    Vec3::new(0.5 - 0.5 * sqrt_3_2, 0.25, 1.0),
                ),
            ],
            Some(&[0, 1, 2, 2, 3, 4, 4, 5, 0, 0, 2, 4]),
        );

        let pipeline =
            create_pipeline(state, &Default::default()).expect("Failed to create polygon pipeline");

        Self {
            triangle,
            quad,
            hexagon,
            pipeline,
        }
    }
}

#[must_use]
#[derive(Clone)]
pub struct Polygon {
    library: ShapesLibrary,
    geometry: Mesh,
    pipeline: wgpu::RenderPipeline,
    xform: Affine3A,
}

impl Element for Polygon {
    type Attribute = ();

    fn state(&self) -> &ShapesState {
        &self.library.state
    }

    fn vertices(&self) -> Mesh {
        self.geometry.clone()
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
    fn polygon(&self, mesh: Mesh) -> Polygon {
        Polygon {
            library: self.clone(),
            geometry: mesh,
            pipeline: self.polygon.pipeline.clone(),
            xform: Affine3A::IDENTITY,
        }
    }

    pub fn triangle(&self, a: impl Position, b: impl Position, c: impl Position) -> Polygon {
        self.polygon(self.polygon.triangle.clone())
            .transform(Mat3::from_cols(a.to_xyz(), b.to_xyz(), c.to_xyz()))
    }

    pub fn unit_quad(&self) -> Polygon {
        self.polygon(self.polygon.quad.clone())
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
        self.polygon(self.polygon.hexagon.clone())
    }
}

impl Debug for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Polygon<{}>", self.geometry.count())
    }
}
