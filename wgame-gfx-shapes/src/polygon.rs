use std::{
    fmt::{self, Debug},
    marker::PhantomData,
};

use glam::{Affine2, Affine3A, Mat3, Vec2, Vec3, Vec4};
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
    pub four_point_quad: Mesh,
    pub hexagon: Mesh,
    pub fill: crate::pipeline::Pipelines,
}

impl PolygonLibrary {
    pub fn new(state: &ShapesState) -> Self {
        let triangle = Mesh::from_arrays(
            state,
            &[
                Vertex::new(Vec4::new(1.0, 0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 1.0))
                    .with_normal(Vec3::ONE),
                Vertex::new(Vec4::new(0.0, 1.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 1.0))
                    .with_normal(Vec3::ONE),
                Vertex::new(Vec4::new(0.0, 0.0, 1.0, 1.0), Vec3::new(0.0, 0.0, 1.0))
                    .with_normal(Vec3::ONE),
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

        // Four affinely independent reference points let an Affine3A map each
        // corner independently. Planar destinations flatten this mesh without
        // introducing perspective interpolation or changing clip-space w.
        let four_point_quad = Mesh::from_arrays(
            state,
            &[
                Vertex::new(Vec3::ZERO.extend(1.0), Vec3::new(0.0, 0.0, 1.0)),
                Vertex::new(Vec3::X.extend(1.0), Vec3::new(1.0, 0.0, 1.0)),
                Vertex::new(Vec3::Y.extend(1.0), Vec3::new(1.0, 1.0, 1.0)),
                Vertex::new(Vec3::Z.extend(1.0), Vec3::new(0.0, 1.0, 1.0)),
            ],
            Some(&[0, 1, 2, 0, 2, 3]),
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
            four_point_quad,
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
    fill: crate::pipeline::Pipelines,
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
            depth: Default::default(),
        }
    }
}

impl_transformable!(Polygon, xform);

#[must_use]
#[derive(Clone)]
pub struct PolygonFill {
    depth: wgame_gfx::DepthMode,
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
            bindings: Vec::new(),
            pipeline: self.shape.fill.get(self.depth),
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
    /// Draw an indexed or non-indexed triangle mesh through the shared shape
    /// renderer. Vertex positions may be 2D, 3D, or homogeneous coordinates;
    /// local coordinates supply UVs. Meshes and pipelines are shared by clones.
    pub fn mesh(&self, mesh: Mesh) -> Polygon {
        Polygon {
            library: self.clone(),
            geometry: mesh,
            fill: self.polygon.fill.clone(),
            xform: Affine3A::IDENTITY,
        }
    }

    pub fn triangle(&self, a: impl Position, b: impl Position, c: impl Position) -> Polygon {
        self.mesh(self.polygon.triangle.clone())
            .transform(Mat3::from_cols(a.to_xyz(), b.to_xyz(), c.to_xyz()))
    }

    pub fn unit_quad(&self) -> Polygon {
        self.mesh(self.polygon.quad.clone())
    }

    /// A quadrilateral with corners in perimeter order and texture coordinates
    /// `(0, 0)`, `(1, 0)`, `(1, 1)`, `(0, 1)`, respectively.
    ///
    /// Draws triangles `abc` and `acd`, with linear texture interpolation within
    /// each triangle. Convex planar corners give an ordinary filled quad;
    /// collapsed edges are allowed. Concave, crossed, or nonplanar corners retain
    /// this fixed triangulation. Geometry is shared between all calls.
    ///
    /// ```
    /// # fn draw(shapes: &wgame_gfx_shapes::ShapesLibrary, scene: &mut wgame_gfx::Scene) {
    /// use glam::Vec2;
    /// use wgame_gfx_shapes::prelude::*;
    /// scene.add(&shapes.quad(
    ///     Vec2::new(10.0, 10.0), Vec2::new(80.0, 20.0),
    ///     Vec2::new(60.0, 70.0), Vec2::new(20.0, 50.0),
    /// ).fill_color(wgame_gfx::types::color::WHITE));
    /// # }
    /// ```
    ///
    /// # Panics
    /// Panics if coordinates or their affine differences are non-finite.
    pub fn quad(
        &self,
        a: impl Position,
        b: impl Position,
        c: impl Position,
        d: impl Position,
    ) -> Polygon {
        self.mesh(self.polygon.four_point_quad.clone())
            .transform(quad_transform([
                a.to_xyz(),
                b.to_xyz(),
                c.to_xyz(),
                d.to_xyz(),
            ]))
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

    /// A 2D line segment with flat ends and the given full width in world units.
    /// Fill and transform it like any other polygon. A zero width or coincident
    /// endpoints draws nothing. No vertex buffers are allocated per line.
    ///
    /// ```
    /// # fn draw(shapes: &wgame_gfx_shapes::ShapesLibrary, scene: &mut wgame_gfx::Scene) {
    /// use wgame_gfx_shapes::prelude::*;
    /// let line = shapes.line(glam::Vec2::ZERO, glam::Vec2::new(30.0, 20.0), 2.0);
    /// scene.add(&line.fill_color(wgame_gfx::types::color::WHITE));
    /// # }
    /// ```
    ///
    /// # Panics
    /// Panics if endpoints or width are non-finite, or width is negative.
    pub fn line(&self, start: Vec2, end: Vec2, width: f32) -> Polygon {
        self.unit_quad()
            .transform(line_transform(start, end, width))
    }

    pub fn unit_hexagon(&self) -> Polygon {
        self.mesh(self.polygon.hexagon.clone())
    }
}

impl Debug for Polygon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Polygon<{}>", self.geometry.count())
    }
}

pub(crate) fn quad_transform([a, b, c, d]: [Vec3; 4]) -> Affine3A {
    let transform = Affine3A::from_mat3_translation(Mat3::from_cols(b - a, c - a, d - a), a);
    assert!(
        transform.is_finite(),
        "quad coordinates and differences must be finite"
    );
    transform
}

fn line_transform(start: Vec2, end: Vec2, width: f32) -> Affine2 {
    assert!(
        start.is_finite() && end.is_finite(),
        "line endpoints must be finite"
    );
    assert!(
        width.is_finite() && width >= 0.0,
        "line width must be finite and nonnegative"
    );
    let half = 0.5 * end - 0.5 * start;
    Affine2::from_cols(
        half,
        half.normalize_or_zero().perp() * (0.5 * width),
        0.5 * start + 0.5 * end,
    )
}

impl PolygonFill {
    pub(crate) fn depth_mode(&self) -> wgame_gfx::DepthMode {
        self.depth
    }
    pub(crate) fn instance_data(&self) -> InstanceData<()> {
        InstanceData {
            matrix: self.shape.xform.to_mat4(),
            tex: self.texture.attribute(),
            custom: (),
        }
    }

    /// Select depth testing/writing; the default is `ReadWrite`.
    pub fn depth(&self, depth: wgame_gfx::DepthMode) -> Self {
        Self {
            depth,
            ..self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{line_transform, quad_transform};
    use glam::{Vec2, Vec3};

    #[test]
    fn quad_maps_all_corners_without_perspective() {
        for corners in [
            [
                Vec3::new(2.0, 3.0, 0.0),
                Vec3::new(20.0, 4.0, 0.0),
                Vec3::new(15.0, 12.0, 0.0),
                Vec3::new(6.0, 10.0, 0.0),
            ],
            [Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::new(0.5, 0.5, 2.0)],
            [Vec3::ZERO, Vec3::X, Vec3::X, Vec3::Y],
        ] {
            let transform = quad_transform(corners);
            for (reference, expected) in [Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z]
                .into_iter()
                .zip(corners)
            {
                let actual = glam::Mat4::from(transform) * reference.extend(1.0);
                assert!((actual.truncate() - expected).length() < 1e-6);
                assert_eq!(actual.w, 1.0);
            }
        }
    }

    #[test]
    fn line_has_flat_ends_and_full_width_in_every_direction() {
        for end in [
            Vec2::new(14.0, 4.0),
            Vec2::new(4.0, 14.0),
            Vec2::new(-2.0, -4.0),
        ] {
            let start = Vec2::splat(4.0);
            for (start, end) in [(start, end), (end, start)] {
                let transform = line_transform(start, end, 6.0);
                assert!((transform.transform_point2(Vec2::NEG_X) - start).length() < 0.0001);
                assert!((transform.transform_point2(Vec2::X) - end).length() < 0.0001);
                let width = transform.transform_vector2(Vec2::Y * 2.0);
                assert!((width.length() - 6.0).abs() < 0.0001);
                assert!(width.dot(end - start).abs() < 0.0001);
            }
        }
    }

    #[test]
    fn empty_lines_have_finite_degenerate_geometry() {
        for (end, width) in [(Vec2::ZERO, 8.0), (Vec2::X, 0.0)] {
            let transform = line_transform(Vec2::ZERO, end, width);
            assert!(transform.is_finite());
            assert_eq!(transform.matrix2.determinant(), 0.0);
        }
    }
}
