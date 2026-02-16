//! 2D graphics primitives for wgame.
//!
//! This crate provides GPU-accelerated rendering of common 2D shapes like circles
//! and polygons. It uses the wgpu graphics API and integrates with wgame-gfx for
//! scene management and wgame-gfx-texture for texturing.
//!
//! # Core Concepts
//!
//! ## Shapes Library
//!
//! The [`ShapesLibrary`] type is the main entry point for rendering shapes. It
//! manages GPU pipelines and resources for efficient rendering.
//!
//! ```no_run
//! # use wgame_gfx::{Graphics, Scene, renderer::Renderer};
//! # use wgame_gfx_texture::TexturingLibrary;
//! # use wgame_gfx_shapes::ShapesLibrary;
//! # async fn example(state: &Graphics, scene: &mut Scene, renderer: &mut Renderer) {
//! let texture_lib = TexturingLibrary::new(state);
//! let shapes_lib = ShapesLibrary::new(state, &texture_lib);
//! # }
//! ```
//!
//! ## Shapes
//!
//! The crate provides two main shape types:
//!
//! - [`Circle`] - A circle or sector shape
//! - [`Polygon`] - A polygon shape with triangle, quad, and hexagon variants
//!
//! ## Fill and Stroke
//!
//! Shapes implement the [`ShapeFill`] and [`ShapeStroke`] traits for rendering
//! filled and stroked versions.
//!
//! ```no_run
//! # use wgame_gfx::{Scene, types::color};
//! # use wgame_gfx_shapes::{ShapesLibrary, ShapeFill, ShapeStroke};
//! # async fn example(shapes_lib: &ShapesLibrary, scene: &mut Scene) {
//! let circle = shapes_lib.unit_circle();
//! // Fill with color
//! scene.add(circle.fill_color(color::RED));
//! // Stroke with color
//! scene.add(circle.stroke_color(2.0, color::BLUE));
//! # }
//! ```
//!
//! ## Textures
//!
//! Shapes can also be rendered with textures using [`Texture`] objects from
//! wgame-gfx-texture.
//!
//! ```no_run
//! # use wgame_gfx::{Scene, types::color};
//! # use wgame_gfx_texture::{TexturingLibrary, TextureSettings};
//! # use wgame_image::{Image, pixel::Pixel};
//! # use wgame_gfx_shapes::{ShapesLibrary, ShapeFill};
//! # use rgb::Rgba;
//! # use half::f16;
//! # async fn example(shapes_lib: &ShapesLibrary, scene: &mut Scene) {
//! # let image = Image::with_data((256, 256), vec![Rgba::new(1.0, 0.0, 0.0, 1.0); 256*256]);
//! let texture_lib = TexturingLibrary::new(shapes_lib.state());
//! let texture = texture_lib.texture(&image, TextureSettings::linear());
//! let circle = shapes_lib.unit_circle();
//! scene.add(circle.fill_texture(&texture));
//! # }
//! ```
//!
//! # Modules
//!
//! - [`shape`] - Core shape traits ([`Shape`], [`ShapeFill`], [`ShapeStroke`], [`Textured`])
//! - [`circle`] - Circle and sector rendering
//! - [`polygon`] - Polygon rendering with triangle, quad, and hexagon primitives
//! - [`geometry`] - Mesh geometry types
//! - [`render`] - Rendering infrastructure
//! - [`shader`] - Shader configuration

#![forbid(unsafe_code)]

mod circle;
pub mod geometry;
mod pipeline;
mod polygon;
mod render;
pub mod shader;
mod shape;

use core::ops::Deref;
use wgame_gfx::{
    Graphics,
    types::{Color, color},
};
use wgame_gfx_texture::{Texture, TexturingLibrary, TexturingState};
use wgame_image::Image;

use crate::{circle::CircleLibrary, polygon::PolygonLibrary};

pub use self::{
    circle::{Circle, CircleFill, CircleStroke},
    geometry::Mesh,
    polygon::{Polygon, PolygonFill},
    shape::{Shape, Textured},
};

/// Module containing convenient trait imports for shape rendering.
pub mod prelude {
    pub use crate::shape::{Shape, ShapeFill, ShapeStroke, Textured};
}

/// Shared state for shape rendering.
///
/// This type contains the GPU resources needed for rendering shapes, including
/// bind group layouts and samplers from the texture system.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ShapesState {
    texture: TexturingState,
}

impl ShapesState {
    /// Returns a reference to the texture state.
    pub fn texture(&self) -> &TexturingState {
        &self.texture
    }
}

impl Deref for ShapesState {
    type Target = Graphics;
    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

/// 2D graphics library for rendering shapes.
///
/// This type provides a convenient interface for creating and rendering 2D
/// shapes. It manages GPU pipelines and a default white texture for coloring.
///
/// # Examples
///
/// ```no_run
/// # use wgame_gfx::{Graphics, Scene};
/// # use wgame_gfx_texture::TexturingLibrary;
/// # use wgame_gfx_shapes::ShapesLibrary;
/// # async fn example(state: &Graphics, scene: &mut Scene) {
/// let texture_lib = TexturingLibrary::new(state);
/// let shapes_lib = ShapesLibrary::new(state, &texture_lib);
/// let circle = shapes_lib.unit_circle();
/// scene.add(circle.fill_color(wgame_gfx::types::color::RED));
/// # }
/// ```
#[derive(Clone)]
pub struct ShapesLibrary {
    state: ShapesState,
    polygon: PolygonLibrary,
    circle: CircleLibrary,
    white_texture: Texture,
}

impl ShapesLibrary {
    /// Creates a new shapes library.
    ///
    /// # Arguments
    ///
    /// * `state` - The graphics state for creating GPU resources.
    /// * `texture` - The texture library for creating textures.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use wgame_gfx::Graphics;
    /// # use wgame_gfx_texture::TexturingLibrary;
    /// # use wgame_gfx_shapes::ShapesLibrary;
    /// # async fn example(state: &Graphics) {
    /// let texture_lib = TexturingLibrary::new(state);
    /// let shapes_lib = ShapesLibrary::new(state, &texture_lib);
    /// # }
    /// ```
    pub fn new(state: &Graphics, texture: &TexturingLibrary) -> Self {
        assert_eq!(state, &**texture.state());
        let state = ShapesState {
            texture: texture.state().clone(),
        };
        Self {
            polygon: PolygonLibrary::new(&state),
            circle: CircleLibrary::new(&state),
            white_texture: texture.texture(
                &Image::with_color((1, 1), color::WHITE.to_rgba_f16()),
                Default::default(),
            ),
            state,
        }
    }

    /// Returns a reference to the shapes state.
    pub fn state(&self) -> &ShapesState {
        &self.state
    }
}
