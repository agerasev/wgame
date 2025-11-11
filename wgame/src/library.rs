use std::ops::Deref;

use anyhow::Result;

use crate::gfx::Graphics;

#[derive(Clone)]
pub struct Library {
    state: Graphics,
    pub texture: crate::texture::TextureLibrary,
    #[cfg(feature = "shapes")]
    pub shapes: crate::shapes::ShapesLibrary,
    #[cfg(feature = "font")]
    pub text: crate::font::TextLibrary,
}

impl Deref for Library {
    type Target = Graphics;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl Library {
    pub fn new(state: &Graphics) -> Result<Self> {
        let state = state.clone();
        let texture = crate::texture::TextureLibrary::new(&state);
        Ok(Self {
            #[cfg(feature = "shapes")]
            shapes: crate::shapes::ShapesLibrary::new(&state, &texture),
            #[cfg(feature = "font")]
            text: crate::font::TextLibrary::new(&texture),
            texture,
            state,
        })
    }
}
