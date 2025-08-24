use core::ops::Deref;

use anyhow::Result;

use crate::gfx::Graphics;

#[derive(Clone)]
pub struct Library {
    state: Graphics,
    #[cfg(feature = "shapes")]
    pub shapes: crate::shapes::ShapeLibrary,
    #[cfg(feature = "text")]
    pub text: crate::text::TextLibrary,
}

impl Deref for Library {
    type Target = Graphics;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl Library {
    pub fn new(state: &Graphics) -> Result<Self> {
        Ok(Self {
            state: state.clone(),
            #[cfg(feature = "shapes")]
            shapes: crate::shapes::ShapeLibrary::new(state)?,
            #[cfg(feature = "text")]
            text: crate::text::TextLibrary::new(state)?,
        })
    }
}
