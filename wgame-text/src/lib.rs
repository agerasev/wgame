#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

mod atlas;
mod renderer;
mod text;

pub use self::{atlas::FontAtlas, renderer::TextRenderer, text::Text};

use alloc::{rc::Rc, vec::Vec};
use core::cell::RefCell;

use anyhow::{Result, anyhow};
use swash::{CacheKey, FontRef, scale::ScaleContext, shape::ShapeContext};

#[derive(Clone)]
pub struct Font {
    data: FontData,
    scale: Rc<RefCell<ScaleContext>>,
    shape: Rc<RefCell<ShapeContext>>,
}

#[derive(Clone)]
pub struct FontData {
    contents: Rc<[u8]>,
    offset: u32,
    key: CacheKey,
}

impl FontData {
    fn new(contents: Vec<u8>, index: usize) -> Result<Self> {
        let contents = Rc::from(contents);
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

impl Font {
    pub fn new(contents: impl Into<Vec<u8>>) -> Result<Self> {
        Ok(Self {
            data: FontData::new(contents.into(), 0)?,
            scale: Rc::new(RefCell::new(ScaleContext::new())),
            shape: Rc::new(RefCell::new(ShapeContext::new())),
        })
    }

    pub fn data(&self) -> &FontData {
        &self.data
    }
}
