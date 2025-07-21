#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

mod atlas;
mod renderer;
mod text;

pub use self::{atlas::FontAtlas, renderer::TextRenderer, text::Text};

use alloc::{rc::Rc, vec::Vec};
use core::cell::RefCell;
use wgame_gfx::registry::{RegistryInit, RegistryKey};

use anyhow::{Result, anyhow};
use swash::{CacheKey, FontRef, scale::ScaleContext, shape::ShapeContext};

#[derive(Clone, Default)]
struct Context {
    scale: Rc<RefCell<ScaleContext>>,
    shape: Rc<RefCell<ShapeContext>>,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct ContextKey;
impl RegistryKey for ContextKey {
    type Value = Context;
}
impl RegistryInit for ContextKey {
    fn create_value(&self, _device: &wgpu::Device) -> Self::Value {
        Context {
            scale: Rc::new(RefCell::new(ScaleContext::new())),
            shape: Rc::new(RefCell::new(ShapeContext::new())),
        }
    }
}

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
