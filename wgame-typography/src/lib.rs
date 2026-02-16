//! Typography rendering utilities for wgame.
//!
//! This crate provides font rasterization and text layout capabilities. It uses
//! the `swash` crate for font rendering and `wgame-image` for atlas management.
//!
//! # Core Concepts
//!
//! ## Font
//!
//! The [`Font`] type represents a font loaded from font data. It can be used to
//! create font atlases and measure text.
//!
//! ```no_run
//! # use wgame_typography::Font;
//! # fn example() -> anyhow::Result<()> {
//! let font = Font::new(std::fs::read("font.ttf")?, 0)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Font Atlas
//!
//! The [`FontAtlas`] type manages a texture atlas for font glyphs. It handles
//! rasterizing glyphs and storing them in a texture.
//!
//! ```no_run
//! # use wgame_typography::{Font, FontAtlas, RasterSettings};
//! # use wgame_image::Atlas;
//! # fn example() -> anyhow::Result<()> {
//! # let font = Font::new(std::fs::read("font.ttf")?, 0)?;
//! let atlas = Atlas::new((256, 256).into());
//! let font_atlas = FontAtlas::new(&atlas, &font, RasterSettings::from(16.0));
//! font_atlas.add_chars("Hello, world!");
//! # Ok(())
//! # }
//! ```
//!
//! ## Text Metrics
//!
//! The [`TextMetrics`] type provides information about text layout, including
//! glyph positions and text width.
//!
//! ```no_run
//! # use wgame_typography::Font;
//! # fn example() -> anyhow::Result<()> {
//! # let font = Font::new(std::fs::read("font.ttf")?, 0)?;
//! let metrics = font.metrics(16.0, "Hello, world!");
//! println!("Text width: {}", metrics.width());
//! # Ok(())
//! # }
//! ```
//!
//! # Modules
//!
//! - [`atlas`] - Font atlas management with [`FontAtlas`]
//! - [`metrics`] - Text metrics with [`TextMetrics`]

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
///
/// This type wraps font data and provides access to the underlying `swash` font
/// reference. It is used to create font atlases and measure text.
///
/// # Examples
///
/// ```no_run
/// # use wgame_typography::Font;
/// # fn example() -> anyhow::Result<()> {
/// let font = Font::new(std::fs::read("font.ttf")?, 0)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Font {
    contents: Rc<[u8]>,
    offset: u32,
    key: CacheKey,
}

impl Font {
    /// Creates a new font from font data.
    ///
    /// # Arguments
    ///
    /// * `contents` - The font data (e.g., contents of a `.ttf` or `.otf` file).
    /// * `index` - The index of the font face in the font data (use 0 for most files).
    ///
    /// # Errors
    ///
    /// Returns an error if the font data is invalid or the index is out of range.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use wgame_typography::Font;
    /// # fn example() -> anyhow::Result<()> {
    /// let font = Font::new(std::fs::read("font.ttf")?, 0)?;
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// # Arguments
    ///
    /// * `size` - The font size in pixels.
    /// * `text` - The text to measure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use wgame_typography::Font;
    /// # fn example() -> anyhow::Result<()> {
    /// # let font = Font::new(std::fs::read("font.ttf")?, 0)?;
    /// let metrics = font.metrics(16.0, "Hello, world!");
    /// println!("Width: {}", metrics.width());
    /// # Ok(())
    /// # }
    /// ```
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
