//! Connected, variable-width ribbons made from shared quad instances.

use wgame_gfx_texture::{SampledTexture, TextureSample};

mod geometry;

use std::{marker::PhantomData, rc::Rc};

use glam::{Affine3A, Vec2};
use wgame_gfx::{
    Camera, Instance, InstanceVisitor, Object, delegate_transformable, impl_transformable,
};

use crate::{
    Shape, ShapesLibrary, impl_textured,
    render::{ShapeResource, ShapeStorage},
    shader::InstanceData,
    shape::ShapeFill,
};

use geometry::Piece;

/// A polyline junction. Width is the full width in local world units, like
/// [`ShapesLibrary::line`]; zero width produces a tapered tip or pinch.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PolylinePoint {
    pub position: Vec2,
    pub width: f32,
}

/// An open ribbon with flat caps and miter joins, falling back to bevel joins
/// when the outer miter exceeds four times the half-width.
///
/// Junction `i` has UVs `(i / (n - 1), 0)` on its left boundary and
/// `(i / (n - 1), 1)` on its right boundary. Left means the perpendicular
/// `(-direction.y, direction.x)`. UV progress uses input indices, not length;
/// extra bevel vertices keep the junction's U. Interpolation is linear within
/// each triangle. Widths and texture coordinates are defined before transforms.
///
/// Consecutive coincident positions keep the last point's width and original U.
/// Fewer than two distinct consecutive positions draw nothing. Reversals split
/// the ribbon into flat-ended runs. At short, wide bends whose inner intersection
/// lies beyond a neighboring point, the inner join falls back to the junction.
/// Self-intersections and overlapping runs are composited normally, not unioned
/// into a single filled path.
///
/// Geometry is tessellated on the CPU at construction and shared by clones.
/// Segments and bevels instance a common quad mesh; construction allocates no GPU
/// buffers. Fill and transform the ribbon like other shapes.
#[must_use]
#[derive(Clone)]
pub struct Polyline {
    library: ShapesLibrary,
    pieces: Rc<[Piece]>,
    xform: Affine3A,
}

impl Shape for Polyline {
    fn library(&self) -> &ShapesLibrary {
        &self.library
    }
}

impl ShapeFill for Polyline {
    type Fill = PolylineFill;

    fn fill_texture(&self, texture: &dyn SampledTexture) -> Self::Fill {
        PolylineFill {
            shape: self.clone(),
            texture: texture.sample(),
            depth: Default::default(),
        }
    }
}

impl_transformable!(Polyline, xform);

/// A filled [`Polyline`], with the usual color and texture modifiers.
#[must_use]
#[derive(Clone)]
pub struct PolylineFill {
    depth: wgame_gfx::DepthMode,
    shape: Polyline,
    texture: TextureSample,
}

impl Instance for PolylineFill {
    type Context = Camera;
    type Resource = ShapeResource<()>;
    type Storage = ShapeStorage<()>;

    fn resource(&self) -> Self::Resource {
        let library = &self.shape.library;
        ShapeResource {
            vertices: library.polygon.four_point_quad.clone(),
            texture: self.texture.resource(),
            bindings: Vec::new(),
            pipeline: library.polygon.fill.get(self.depth),
            device: library.state().device().clone(),
            _ghost: PhantomData,
        }
    }

    fn new_storage(&self) -> Self::Storage {
        ShapeStorage::new(self.resource())
    }

    fn store(&self, storage: &mut Self::Storage) {
        let texture = self.texture.attribute();
        for piece in self.shape.pieces.iter() {
            storage.instances.push(InstanceData {
                matrix: (self.shape.xform * piece.xform).into(),
                tex: texture.map_coord(piece.texcoord),
                custom: (),
            });
        }
    }
}

impl Object for PolylineFill {
    type Context = Camera;

    fn for_each_instance<V: InstanceVisitor<Camera>>(&self, visitor: &mut V) {
        // An empty ribbon must not create a batch with an empty vertex buffer.
        if !self.shape.pieces.is_empty() {
            visitor.visit(self);
        }
    }
}
delegate_transformable!(PolylineFill, shape);
impl_textured!(PolylineFill, texture);

impl ShapesLibrary {
    /// Construct an open, variable-width [`Polyline`] with miter limit 4.
    ///
    /// ```
    /// # fn draw(shapes: &wgame_gfx_shapes::ShapesLibrary, scene: &mut wgame_gfx::Scene) {
    /// use glam::Vec2;
    /// use wgame_gfx_shapes::{PolylinePoint, prelude::*};
    /// let ribbon = shapes.polyline(&[
    ///     PolylinePoint { position: Vec2::new(10.0, 20.0), width: 12.0 },
    ///     PolylinePoint { position: Vec2::new(50.0, 20.0), width: 6.0 },
    ///     PolylinePoint { position: Vec2::new(70.0, 50.0), width: 0.0 },
    /// ]);
    /// scene.add(&ribbon.fill_color(wgame_gfx::types::color::WHITE));
    /// # }
    /// ```
    ///
    /// # Panics
    /// Panics for non-finite positions or widths, negative widths, or generated
    /// geometry whose coordinates or affine differences do not fit finite f32.
    pub fn polyline(&self, points: &[PolylinePoint]) -> Polyline {
        Polyline {
            library: self.clone(),
            pieces: geometry::tessellate(points).into(),
            xform: Affine3A::IDENTITY,
        }
    }
}

impl PolylineFill {
    /// Select depth testing/writing; the default is `ReadWrite`.
    pub fn depth(&self, depth: wgame_gfx::DepthMode) -> Self {
        Self {
            depth,
            ..self.clone()
        }
    }
}
