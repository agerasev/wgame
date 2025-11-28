use std::ops::Deref;

#[cfg(feature = "shapes")]
use crate::shapes::ShapesLibrary;
#[cfg(feature = "typography")]
use crate::typography::{Font, FontTexture, TypographyLibrary};
use crate::{gfx::Graphics, texture::TextureLibrary};

#[derive(Clone)]
pub struct Library {
    state: Graphics,
    texture: TextureLibrary,
    #[cfg(feature = "shapes")]
    shapes: ShapesLibrary,
    #[cfg(feature = "typography")]
    typography: TypographyLibrary,
}

impl Deref for Library {
    type Target = TextureLibrary;

    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

impl Library {
    pub fn new(state: &Graphics) -> Self {
        let state = state.clone();
        let texture = TextureLibrary::new(&state);
        Self {
            #[cfg(feature = "shapes")]
            shapes: ShapesLibrary::new(&state, &texture),
            #[cfg(feature = "typography")]
            typography: TypographyLibrary::new(&texture),
            texture,
            state,
        }
    }

    pub fn state(&self) -> &Graphics {
        &self.state
    }

    #[cfg(feature = "shapes")]
    pub fn shapes(&self) -> &ShapesLibrary {
        &self.shapes
    }

    #[cfg(feature = "typography")]
    pub fn font(&self, font: &Font, size: f32) -> FontTexture {
        self.typography.texture(font, size)
    }
}
