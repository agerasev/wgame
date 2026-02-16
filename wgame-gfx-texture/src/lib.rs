//! Texture rendering utilities for wgame.
//!
//! This crate provides texture handling and rendering capabilities built on top
//! of `wgpu`. It includes support for texture atlases, texture filtering, and
//! coordinate transformations.
//!
//! # Core Concepts
//!
//! ## Texture Atlas
//!
//! The [`TextureAtlas`] type provides a way to pack multiple textures into a
//! single GPU texture for efficient rendering. Textures are allocated from the
//! atlas and can be updated dynamically.
//!
//! ```no_run
//! # use wgame_gfx_texture::{TexturingLibrary, TextureSettings};
//! # use wgame_gfx::Graphics;
//! # async fn example(state: &Graphics) {
//! let library = TexturingLibrary::new(state);
//! // Allocate a texture from the atlas
//! # }
//! ```
//!
//! ## Texture
//!
//! The [`Texture`] type represents a single texture within an atlas. It provides
//! methods for updating the texture data, resizing, and transforming coordinates.
//!
//! ## Texturing Library
//!
//! The [`TexturingLibrary`] type provides a convenient way to create and manage
//! textures. It includes utilities for creating gradients and loading images.
//!
//! ```no_run
//! # use wgame_gfx_texture::TexturingLibrary;
//! # use wgame_gfx::Graphics;
//! # use wgame_image::Image;
//! # use rgb::Rgba;
//! # use half::f16;
//! # async fn example(state: &Graphics) {
//! # let image = Image::with_data((256, 256), vec![Rgba::new(1.0, 0.0, 0.0, 1.0); 256*256]);
//! let library = TexturingLibrary::new(state);
//! let texture = library.texture(&image, TextureSettings::linear());
//! # }
//! ```
//!
//! # Modules
//!
//! - [`state`] - Shared texturing state with bind group layouts and samplers
//! - [`texel`] - Texel trait for pixel types
//! - [`texture`] - Texture, atlas, and resource types

#![forbid(unsafe_code)]

mod state;
mod texel;
mod texture;

use glam::{Affine2, Vec2};
use half::f16;
use rgb::Rgba;
use wgame_gfx::{Graphics, types::Color};
use wgame_image::{Image, ImageBase, ImageWriteMut};

pub use self::{
    state::TexturingState,
    texel::Texel,
    texture::{
        FilterMode, Texture, TextureAtlas, TextureAttribute, TextureResource, TextureSettings,
    },
};

/// A library for managing textures.
///
/// This type provides a convenient interface for creating and managing textures
/// using a texture atlas. It handles the creation of bind groups, samplers, and
/// other GPU resources needed for texture rendering.
///
/// # Examples
///
/// ```no_run
/// # use wgame_gfx_texture::TexturingLibrary;
/// # use wgame_gfx::Graphics;
/// # async fn example(state: &Graphics) {
/// let library = TexturingLibrary::new(state);
/// // Use the library to create textures
/// # }
/// ```
#[derive(Clone)]
pub struct TexturingLibrary {
    state: TexturingState,
    default_atlas: TextureAtlas,
}

impl TexturingLibrary {
    /// Creates a new texture library.
    ///
    /// # Arguments
    ///
    /// * `state` - The graphics state to use for creating GPU resources.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use wgame_gfx_texture::TexturingLibrary;
    /// # use wgame_gfx::Graphics;
    /// # async fn example(state: &Graphics) {
    /// let library = TexturingLibrary::new(state);
    /// # }
    /// ```
    pub fn new(state: &Graphics) -> Self {
        let state = TexturingState::new(state);
        Self {
            default_atlas: TextureAtlas::new(
                &state,
                Default::default(),
                wgpu::TextureFormat::Rgba16Float,
            ),
            state,
        }
    }

    /// Returns the texturing state.
    pub fn state(&self) -> &TexturingState {
        &self.state
    }

    /// Creates a texture from an image.
    ///
    /// # Arguments
    ///
    /// * `image` - The image data to upload to the texture.
    /// * `settings` - The texture settings for filtering.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use wgame_gfx_texture::{TexturingLibrary, TextureSettings};
    /// # use wgame_gfx::Graphics;
    /// # use wgame_image::Image;
    /// # use rgb::Rgba;
    /// # use half::f16;
    /// # async fn example(state: &Graphics) {
    /// # let image = Image::with_data((256, 256), vec![Rgba::new(1.0, 0.0, 0.0, 1.0); 256*256]);
    /// let library = TexturingLibrary::new(state);
    /// let texture = library.texture(&image, TextureSettings::linear());
    /// # }
    /// ```
    pub fn texture(&self, image: &Image<Rgba<f16>>, settings: TextureSettings) -> Texture {
        let texture = self.default_atlas.allocate(image.size(), settings);
        texture.update(|mut dst| dst.copy_from(image));
        texture
    }

    /// Creates a 1D gradient texture from an array of colors.
    ///
    /// # Arguments
    ///
    /// * `colors` - An array of colors to interpolate between.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use wgame_gfx_texture::TexturingLibrary;
    /// # use wgame_gfx::Graphics;
    /// # use wgame_gfx::types::Color;
    /// # async fn example(state: &Graphics) {
    /// let library = TexturingLibrary::new(state);
    /// let gradient = library.gradient([wgame_gfx::types::color::RED, wgame_gfx::types::color::BLUE]);
    /// # }
    /// ```
    pub fn gradient<T: Color, const N: usize>(&self, colors: [T; N]) -> Texture {
        self.gradient2([colors])
    }

    /// Creates a 2D gradient texture from a 2D array of colors.
    ///
    /// # Arguments
    ///
    /// * `colors` - A 2D array of colors to interpolate between.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use wgame_gfx_texture::TexturingLibrary;
    /// # use wgame_gfx::Graphics;
    /// # use wgame_gfx::types::Color;
    /// # async fn example(state: &Graphics) {
    /// let library = TexturingLibrary::new(state);
    /// let gradient = library.gradient2([[wgame_gfx::types::color::RED, wgame_gfx::types::color::BLUE]]);
    /// # }
    /// ```
    pub fn gradient2<T: Color, const M: usize, const N: usize>(
        &self,
        colors: [[T; M]; N],
    ) -> Texture {
        let colors = colors
            .into_iter()
            .flatten()
            .map(|c| c.to_rgba_f16())
            .collect::<Vec<_>>();
        let pix_size = Vec2::new(M as f32, N as f32).recip();
        self.texture(
            &Image::with_data((M as u32, N as u32), colors),
            TextureSettings::linear(),
        )
        .transform_coord(Affine2::from_scale_angle_translation(
            1.0 - pix_size,
            0.0,
            0.5 * pix_size,
        ))
    }
}
