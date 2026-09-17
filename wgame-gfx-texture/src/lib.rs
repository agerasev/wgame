//! CPU texture atlases, GPU render textures, and shared sampling for wgpu.
//!
//! [`Texture`] owns CPU-editable pixels through a [`TextureAtlas`], uploading them
//! on use. [`RenderTexture`] implements [`wgame_gfx::Target`] and owns a dedicated
//! GPU allocation; its explicit asynchronous readback produces a detached image.
//! Both implement [`SampledTexture`], accepted by shapes and normal-map materials.
//! Neither type changes ownership mode, and no CPU mirror is kept for GPU rendering.
//!
//! ```no_run
//! # async fn preview(graphics: &wgame_gfx::Graphics) -> Result<(), Box<dyn std::error::Error>> {
//! use wgame_gfx::{Target, types::color};
//! use wgame_gfx_texture::{SampledTexture, TexturingLibrary, TextureSettings};
//! let textures = TexturingLibrary::new(graphics);
//! let mut preview = textures.render_texture((256, 256), TextureSettings::linear())?;
//! preview.clear(color::BLUE);
//! // Draw through Target::render, Target::scene, or borrowed viewports as usual.
//! preview.submit(); // Submit drawing before another target samples it.
//! let sampled = preview.sample(); // Can outlive the drawing target.
//! let pixels = preview.readback().await?; // Detached snapshot; no automatic downloads.
//! let frozen = textures.texture(&pixels, TextureSettings::linear());
//! # Ok(()) }
//! ```
//!
//! See [`Texture`] for atlas updates/relocation, [`RenderTexture`] for submission
//! and snapshot contracts, and [`TextureSample`] for retained sampling handles.

#![forbid(unsafe_code)]

mod gpu;
mod readback;
mod render_texture;
mod sampling;
mod state;
mod texel;
mod texture;

use glam::{Affine2, Vec2};
use half::f16;
use rgb::Rgba;
use wgame_gfx::{Graphics, types::Color};
use wgame_image::{Image, ImageBase, ImageWriteMut};

pub use self::{
    readback::ReadbackError,
    render_texture::{RenderTexture, RenderTextureError},
    sampling::{SampledTexture, TextureAttribute, TextureResource, TextureSample},
    state::TexturingState,
    texel::Texel,
    texture::{FilterMode, Texture, TextureAtlas, TextureSettings},
};

/// A library for managing textures.
#[derive(Clone)]
pub struct TexturingLibrary {
    state: TexturingState,
    default_atlas: TextureAtlas,
}

impl TexturingLibrary {
    /// Creates a new texture library.
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

    /// Create a GPU render target that implements the same sampling trait as images.
    pub fn render_texture(
        &self,
        size: (u32, u32),
        settings: TextureSettings,
    ) -> Result<RenderTexture, RenderTextureError> {
        RenderTexture::new(&self.state, size, settings)
    }

    /// Creates a texture from an image.
    pub fn texture(&self, image: &Image<Rgba<f16>>, settings: TextureSettings) -> Texture {
        let texture = self.default_atlas.allocate(image.size(), settings);
        texture.update(|mut dst| dst.copy_from(image));
        texture
    }

    /// Creates a 1D gradient texture from an array of colors.
    pub fn gradient<T: Color, const N: usize>(&self, colors: [T; N]) -> Texture {
        self.gradient2([colors])
    }

    /// Creates a 2D gradient texture from a 2D array of colors.
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
