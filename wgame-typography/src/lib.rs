//! Font rasterization and text layout utilities.
//!
//! Provides font loading, glyph rasterization, and text metrics using swash.

#![forbid(unsafe_code)]

mod atlas;
mod metrics;

pub use self::atlas::{FontAtlas, GlyphImageInfo, RasterSettings};
pub use metrics::TextMetrics;
pub use swash;

use std::rc::Rc;

use anyhow::{Result, anyhow};
use swash::{CacheKey, FontRef};

/// A font loaded from font data.
#[derive(Clone)]
pub struct Font {
    contents: Rc<[u8]>,
    offset: u32,
    key: CacheKey,
}

impl Font {
    /// Creates a new font from font data.
    pub fn new(contents: impl Into<Vec<u8>>, index: usize) -> Result<Self> {
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

    /// Measures the given text at the specified size.
    pub fn metrics<T: Into<String>>(&self, size: f32, text: T) -> TextMetrics {
        TextMetrics::new(self, size, text)
    }

    /// Returns a font reference for use with `swash`.
    pub fn as_ref(&'_ self) -> FontRef<'_> {
        FontRef {
            data: &self.contents,
            offset: self.offset,
            key: self.key,
        }
    }
}
