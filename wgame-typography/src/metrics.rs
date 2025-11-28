use std::cell::RefCell;
use swash::shape::{ShapeContext, cluster::Glyph};

use crate::Font;

thread_local! {
    static CONTEXT: RefCell<ShapeContext> = Default::default();
}

pub struct TextMetrics {
    text: String,
    size: f32,
    glyphs: Vec<Glyph>,
}

impl TextMetrics {
    pub fn new<T: Into<String>>(font: &Font, size: f32, text: T) -> Self {
        let text = text.into();
        let glyphs = CONTEXT.with_borrow_mut(|context| {
            let mut shaper = context.builder(font.as_ref()).size(size).build();
            shaper.add_str(&text);
            let mut glyphs = Vec::new();
            shaper.shape_with(|cluster| {
                for glyph in cluster.glyphs {
                    glyphs.push(*glyph);
                }
            });
            glyphs
        });
        Self { text, size, glyphs }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn size(&self) -> f32 {
        self.size
    }
    pub fn glyphs(&self) -> &[Glyph] {
        &self.glyphs
    }

    pub fn width(&self) -> f32 {
        let mut advance = 0.0;
        for glyph in &self.glyphs {
            advance += glyph.advance;
        }
        advance
    }
}
