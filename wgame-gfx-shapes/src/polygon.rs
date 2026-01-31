use std::{
    fmt::{self, Debug},
    marker::PhantomData,
};

use glam::{Affine3A, Mat3, Vec2, Vec3, Vec4};
use wgame_gfx::{
    Camera, Instance, Object, delegate_transformable, impl_object_for_instance, impl_transformable,
    modifiers::Transformable,
    types::{Position, Transform},
};
use wgame_gfx_texture::Texture;

use crate::{
    Mesh, Shape, ShapesLibrary, ShapesState, impl_textured,
    pipeline::create_pipeline,
    render::{ShapeResource, ShapeStorage},
    shader::{InstanceData, Vertex},
    shape::ShapeFill,
};

#[derive(Clone)]
pub struct PolygonLibrary {
    pub triangle: Mesh,
    pub quad: Mesh,
    pub hexagon: Mesh,
    pub fill: wgpu::RenderPipeline,
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
            fill: pipeline,
        }
    }
}

#[must_use]
#[derive(Clone)]
pub struct Polygon {
    library: ShapesLibrary,
    geometry: Mesh,
    fill: wgpu::RenderPipeline,
    xform: Affine3A,
}

impl Shape for Polygon {
    fn library(&self) -> &ShapesLibrary {
        &self.library
    }
}

impl ShapeFill for Polygon {
    type Fill = PolygonFill;

    fn fill_texture(&self, texture: &Texture) -> Self::Fill {
        PolygonFill {
            shape: self.clone(),
            texture: texture.clone(),
        }
    }
}

impl_transformable!(Polygon, xform);

#[must_use]
#[derive(Clone)]
pub struct PolygonFill {
    shape: Polygon,
    texture: Texture,
}

impl Instance for PolygonFill {
    type Context = Camera;
    type Resource = ShapeResource<()>;
    type Storage = ShapeStorage<()>;

    fn resource(&self) -> Self::Resource {
        ShapeResource {
            vertices: self.shape.geometry.clone(),
            texture: self.texture.resource(),
            uniforms: None,
            pipeline: self.shape.fill.clone(),
            device: self.shape.library.state().device().clone(),
            _ghost: PhantomData,
        }
    }

    fn new_storage(&self) -> Self::Storage {
        ShapeStorage::new(self.resource())
    }

    fn store(&self, storage: &mut Self::Storage) {
        storage.instances.push(InstanceData {
            matrix: self.shape.xform.to_mat4(),
            tex: self.texture.attribute(),
            custom: (),
        });
    }
}

impl_object_for_instance!(PolygonFill);
delegate_transformable!(PolygonFill, shape);
impl_textured!(PolygonFill, texture);

impl ShapesLibrary {
    fn polygon(&self, mesh: Mesh) -> Polygon {
        Polygon {
            library: self.clone(),
            geometry: mesh,
            fill: self.polygon.fill.clone(),
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
