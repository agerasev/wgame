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
    pub fn shapes(&self) -> &ShapesLibrary {
        &self.shapes
    }
    #[cfg(feature = "typography")]
    pub fn typography(&self) -> &TypographyLibrary {
        &self.typography
    }

    #[cfg(feature = "image")]
    pub fn make_texture(&self, image: &Image<Rgba<f16>>, settings: TextureSettings) -> Texture {
        self.texturing.texture(image, settings)
    }
    #[cfg(all(feature = "fs", feature = "image"))]
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
    pub async fn load_font(&self, path: impl AsRef<Path>) -> Result<Font> {
        Ok(self.make_font(&FontData::new(read_bytes(path).await?, 0)?))
    }
}
