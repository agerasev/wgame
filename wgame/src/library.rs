#[cfg(all(feature = "fs", any(feature = "image", feature = "typography")))]
use anyhow::Result;
#[cfg(feature = "image")]
use half::f16;
#[cfg(feature = "image")]
use rgb::Rgba;
#[cfg(feature = "image")]
use wgame_gfx_texture::TextureSettings;

#[cfg(feature = "image")]
use crate::image::Image;
#[cfg(feature = "shapes")]
use crate::shapes::ShapesLibrary;
#[cfg(feature = "typography")]
use crate::typography::{Font, FontData, TypographyLibrary};
use crate::{gfx::Graphics, texture::TexturingLibrary};

#[cfg(all(feature = "fs", any(feature = "image", feature = "typography")))]
use crate::fs::{Path, read_bytes};
#[cfg(feature = "image")]
use crate::texture::Texture;

/// Shared graphics helpers for reusable shapes, textures, and fonts.
///
/// Create one outside the frame loop. Shapes, text, and textures use cheap shared
/// handles; reuse unchanged geometry and font rasters instead of rebuilding them
/// each frame. File loading uses the optional `fs` module.
#[derive(Clone)]
pub struct Library {
    state: Graphics,
    texturing: TexturingLibrary,
    #[cfg(feature = "shapes")]
    shapes: ShapesLibrary,
    #[cfg(feature = "typography")]
    typography: TypographyLibrary,
}

impl Library {
    pub fn new(state: &Graphics) -> Self {
        let state = state.clone();
        let texture = TexturingLibrary::new(&state);
        Self {
            #[cfg(feature = "shapes")]
            shapes: ShapesLibrary::new(&state, &texture),
            #[cfg(feature = "typography")]
            typography: TypographyLibrary::new(&texture),
            texturing: texture,
            state,
        }
    }

    pub fn state(&self) -> &Graphics {
        &self.state
    }

    pub fn texturing(&self) -> &TexturingLibrary {
        &self.texturing
    }
    #[cfg(feature = "shapes")]
    /// Reusable shape geometry and pipelines (`shapes` feature).
    ///
    /// This example draws in physical pixels using [`crate::gfx::Target::physical_camera`]:
    ///
    /// ```no_run
    /// use wgame::{Library, Window, prelude::*, glam::Vec2, gfx::types::color};
    /// async fn draw(mut window: Window<'_>) -> wgame::Result<()> {
    ///     let library = Library::new(window.graphics());
    ///     let circle = library.shapes().unit_circle().fill_color(color::CYAN);
    ///     while let Some(mut frame) = window.next_frame().await? {
    ///         frame.clear(color::BLACK);
    ///         let camera = frame.physical_camera();
    ///         let mut scene = frame.scene();
    ///         scene.camera = camera;
    ///         scene.add(&circle.scale(40.0).move_to(Vec2::new(100.0, 100.0)));
    ///         scene.render();
    ///         frame.present();
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn shapes(&self) -> &ShapesLibrary {
        &self.shapes
    }
    #[cfg(feature = "typography")]
    pub fn typography(&self) -> &TypographyLibrary {
        &self.typography
    }

    #[cfg(feature = "image")]
    /// Upload an image without automatic sRGB-to-linear conversion (`image` feature).
    /// Choose image conversions and custom target formats deliberately; the default
    /// window surface prefers a non-sRGB format.
    pub fn make_texture(&self, image: &Image<Rgba<f16>>, settings: TextureSettings) -> Texture {
        self.texturing.texture(image, settings)
    }
    #[cfg(all(feature = "fs", feature = "image"))]
    /// Read, decode, and upload a texture (`fs` + `image` features).
    /// Decoding defaults to PNG; see [`crate::fs`] for path semantics.
    pub async fn load_texture(
        &self,
        path: impl AsRef<Path>,
        settings: TextureSettings,
    ) -> Result<Texture> {
        Ok(self.make_texture(&Image::decode_auto(&read_bytes(path).await?)?, settings))
    }

    #[cfg(feature = "typography")]
    pub fn make_font(&self, font: &FontData) -> Font {
        Font::new(&self.typography, font)
    }
    #[cfg(all(feature = "fs", feature = "typography"))]
    /// Read a font for reuse across rasters (`fs` + `typography` features).
    /// See [`crate::fs`] for path semantics and [`crate::typography`] for text limits.
    pub async fn load_font(&self, path: impl AsRef<Path>) -> Result<Font> {
        Ok(self.make_font(&FontData::new(read_bytes(path).await?, 0)?))
    }
}
