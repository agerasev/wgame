#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

use alloc::{rc::Rc, vec::Vec};

use anyhow::{Result, anyhow};
use guillotiere::{Allocation, AtlasAllocator};
use hashbrown::HashMap;
use swash::{
    Attributes, CacheKey, Charmap, FontRef, GlyphId,
    scale::{Render, ScaleContext, Scaler, Source, StrikeWith, image::Image},
};

use wgame_gfx::{Instance, State, Texture};

struct FontData {
    contents: Rc<[u8]>,
    offset: u32,
    key: CacheKey,
}

impl FontData {
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

    pub fn attributes(&self) -> Attributes {
        self.as_ref().attributes()
    }

    pub fn charmap(&self) -> Charmap {
        self.as_ref().charmap()
    }

    pub fn as_ref(&self) -> FontRef {
        FontRef {
            data: &self.contents,
            offset: self.offset,
            key: self.key,
        }
    }
}

pub struct Font {
    data: FontData,
    context: ScaleContext,
}

pub struct FontAtlas<'b> {
    font: &'b FontData,
    scaler: Scaler<'b>,
    render: Render<'b>,
    atlas: AtlasAllocator,
    map: HashMap<GlyphId, Allocation>,
    image: Image,
}

impl<'b> FontAtlas<'b> {
    pub fn new(font: &'b mut Font, size: f32) -> Self {
        let scaler = font
            .context
            .builder(font.data.as_ref())
            .size(size)
            .hint(false)
            .build();
        let render = Render::new(&[
            Source::ColorOutline(0),
            Source::ColorBitmap(StrikeWith::BestFit),
            Source::Outline,
        ]);
        let atlas = AtlasAllocator::new((64, 64).into());
        Self {
            font: &font.data,
            scaler,
            render,
            atlas,
            map: HashMap::default(),
            image: Image::new(),
        }
    }

    fn grow(&mut self) {
        self.atlas.grow(self.atlas.size() * 2);
        todo!("Resize image");
    }

    fn allocate(&mut self, glyph_id: GlyphId, size: (i32, i32)) -> Allocation {
        if let Some(alloc) = self.map.get(&glyph_id) {
            return *alloc;
        }
        loop {
            match self.atlas.allocate(size.into()) {
                Some(alloc) => {
                    self.map.insert(glyph_id, alloc);
                    return alloc;
                }
                None => self.grow(),
            }
        }
    }

    fn add_char(&mut self, codepoint: impl Into<u32>) {
        let glyph_id = self.font.charmap().map(codepoint);
        let size = todo!("Get glyph size");
        let alloc = self.allocate(glyph_id, size);
        self.render
            .render_into(&mut self.scaler, glyph_id, &mut self.image);
    }
}
