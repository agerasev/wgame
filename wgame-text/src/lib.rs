#![forbid(unsafe_code)]

mod atlas;
mod renderer;
mod text;

pub use self::{atlas::FontAtlas, renderer::TextLibrary, text::Text};

use std::rc::Rc;

use anyhow::{Result, anyhow};
use swash::{CacheKey, FontRef};

#[derive(Clone)]
pub struct Font {
    contents: Rc<[u8]>,
    offset: u32,
    key: CacheKey,
}

impl Font {
    fn new(contents: impl Into<Vec<u8>>, index: usize) -> Result<Self> {
        let contents = Rc::from(contents.into());
        let font = FontRef::from_index(&contents, index)
            .ok_or_else(|| anyhow!("Font data validation error"))?;
        let (offset, key) = (font.offset, font.key);
        Ok(Self {
            contents,
            offset,
            key,
        })
    }

    pub fn as_ref(&self) -> FontRef {
        FontRef {
            data: &self.contents,
            offset: self.offset,
            key: self.key,
        }
    }
}
