#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

use alloc::{rc::Rc, vec::Vec};

use anyhow::{Result, anyhow};
use etagere::{AllocId, Allocation, AtlasAllocator};
use hashbrown::HashMap;
use image::{GenericImage, GenericImageView, GrayImage, math::Rect};
use swash::{
    CacheKey, FontRef, GlyphId,
    scale::{Render, ScaleContext, Scaler, Source, StrikeWith},
    zeno::Placement,
};

pub struct Font {
    data: FontData,
    context: ScaleContext,
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
            context: ScaleContext::new(),
        })
    }
}

struct GlyphImageInfo {
    alloc_id: AllocId,
    location: Rect,
    placement: Placement,
}

pub struct FontAtlas<'b> {
    font: FontRef<'b>,
    scaler: Scaler<'b>,
    render: Render<'b>,
    atlas: AtlasAllocator,
    map: HashMap<GlyphId, Option<GlyphImageInfo>>,
    image: GrayImage,
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
        let init_dim = ((4.0 * size).ceil().clamp(u32::MIN as f32, i32::MAX as f32) as u32)
            .next_power_of_two();
        let atlas = AtlasAllocator::new((init_dim as i32, init_dim as i32).into());
        Self {
            font: font.data.as_ref(),
            scaler,
            render,
            atlas,
            map: HashMap::default(),
            image: GrayImage::new(init_dim, init_dim),
        }
    }

    fn grow(&mut self) {
        let mut new_atlas = AtlasAllocator::new(self.atlas.size() * 2);
        let mut new_image = GrayImage::new(2 * self.image.width(), 2 * self.image.height());

        for maybe_info in self.map.values_mut() {
            let info = match maybe_info {
                Some(info) => info,
                None => continue,
            };
            let rect = info.location;
            let alloc = new_atlas
                .allocate((rect.width as i32, rect.height as i32).into())
                .expect("Cannot reallocate glyphs during atlas grow");
            assert!(
                (rect.width <= alloc.rectangle.width() as u32)
                    && (rect.height <= alloc.rectangle.height() as u32),
            );
            let new_rect = Rect {
                x: alloc.rectangle.min.x as u32,
                y: alloc.rectangle.min.y as u32,
                ..rect
            };
            new_image
                .copy_from(
                    &*self.image.view(rect.x, rect.y, rect.width, rect.height),
                    new_rect.x,
                    new_rect.y,
                )
                .expect("Error copying glyphs from one image to another");
            info.alloc_id = alloc.id;
            info.location = new_rect;
        }

        self.atlas = new_atlas;
        self.image = new_image;
    }

    fn alloc_space(&mut self, width: u32, height: u32) -> Allocation {
        loop {
            match self.atlas.allocate((width as i32, height as i32).into()) {
                Some(alloc) => break alloc,
                None => self.grow(),
            }
        }
    }

    pub fn add_char(&mut self, codepoint: impl Into<u32>) {
        let glyph_id = self.font.charmap().map(codepoint);
        if self.map.contains_key(&glyph_id) {
            return;
        }
        let (image, placement) = match self.render.render(&mut self.scaler, glyph_id) {
            Some(img) => (
                GrayImage::from_vec(img.placement.width, img.placement.height, img.data)
                    .expect("Cannot convert glyph image"),
                img.placement,
            ),
            None => {
                assert!(self.map.insert(glyph_id, None).is_none());
                return;
            }
        };
        let alloc = self.alloc_space(placement.width, placement.height);
        let rect = Rect {
            x: alloc.rectangle.min.x as u32,
            y: alloc.rectangle.min.y as u32,
            width: placement.width,
            height: placement.height,
        };
        self.image
            .copy_from(&image, rect.x, rect.y)
            .expect("Error copying glyph image");
        assert!(
            self.map
                .insert(
                    glyph_id,
                    Some(GlyphImageInfo {
                        alloc_id: alloc.id,
                        location: rect,
                        placement
                    })
                )
                .is_none()
        );
    }

    pub fn image(&self) -> &GrayImage {
        &self.image
    }
    pub fn debug_svg(&self) -> Vec<u8> {
        let mut svg_data = Vec::new();
        self.atlas.dump_svg(&mut svg_data).unwrap();
        svg_data
    }
}
