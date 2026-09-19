//! Shared presentation helpers for the browser and desktop examples.
#![forbid(unsafe_code)]

pub mod gallery;

use std::collections::BTreeMap;
use wgame::{
    Library, Result,
    prelude::*,
    typography::{Font, FontData, FontTexture, Text},
};

/// Labels rasterized at their displayed physical size, using the embedded font.
/// Set the number of physical pixels per drawing unit before creating text.
/// For a logical camera this is the display scale; for a fitted gallery it is
/// the camera's scale from gallery coordinates to the physical viewport.
pub struct Labels {
    font: Font,
    pixels_per_unit: f32,
    rasters: BTreeMap<u32, FontTexture>,
}

impl Labels {
    pub fn new(library: &Library) -> Result<Self> {
        Ok(Self {
            font: library.make_font(&FontData::new(
                include_bytes!("../assets/free-sans-bold.ttf").to_vec(),
                0,
            )?),
            pixels_per_unit: 1.0,
            rasters: BTreeMap::new(),
        })
    }

    /// Returns whether cached text needs rebuilding. The scale must be positive
    /// and finite. Old text remains valid, but retains its previous raster.
    pub fn set_scale(&mut self, pixels_per_unit: f32) -> bool {
        assert!(pixels_per_unit.is_finite() && pixels_per_unit > 0.0);
        if self.pixels_per_unit == pixels_per_unit {
            return false;
        }
        self.pixels_per_unit = pixels_per_unit;
        // Release obsolete sizes instead of accumulating atlases while resizing.
        self.rasters.clear();
        true
    }

    /// Create text already scaled to `size` drawing units. Each size has its own
    /// glyph raster; callers can cache unchanged text until the scale changes.
    pub fn text(&mut self, text: &str, size: f32) -> Text {
        let physical_size = size * self.pixels_per_unit;
        assert!(physical_size.is_finite() && physical_size > 0.0);
        self.rasters
            .entry(physical_size.to_bits())
            .or_insert_with(|| self.font.rasterize(physical_size))
            .text(text)
            .scale(size)
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
