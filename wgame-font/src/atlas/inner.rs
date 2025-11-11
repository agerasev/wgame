use etagere::{AllocId, Allocation, AtlasAllocator};
use euclid::default::{Rect, Size2D};
use std::collections::BTreeMap;
use swash::{
    GlyphId,
    scale::{Render, Scaler, Source, StrikeWith},
    zeno::Placement,
};
use wgame_image::{AtlasImage, Image, prelude::*};

#[derive(Clone, Copy, Debug)]
pub(crate) struct GlyphImageInfo {
    pub _alloc_id: AllocId,
    pub location: Rect<u32>,
    pub placement: Placement,
}

pub(crate) struct InnerAtlas {
    allocator: AtlasAllocator,
    mapping: BTreeMap<GlyphId, Option<GlyphImageInfo>>,
    image: AtlasImage<u8>,
    render: Render<'static>,
}

impl InnerAtlas {
    pub fn new(image: AtlasImage<u8>) -> Self {
        Self {
            allocator: AtlasAllocator::new(image.size().cast()),
            mapping: BTreeMap::default(),
            image,
            render: Render::new(&[
                Source::ColorOutline(0),
                Source::ColorBitmap(StrikeWith::BestFit),
                Source::Outline,
            ]),
        }
    }

    fn grow(&mut self, prev_size: Option<Size2D<i32>>) -> Result<(), Size2D<i32>> {
        let new_size = {
            let size = prev_size.unwrap_or_else(|| self.allocator.size());
            if size.width < size.height {
                Size2D::new(size.width.checked_mul(2).unwrap(), size.height)
            } else {
                Size2D::new(size.width, size.height.checked_mul(2).unwrap())
            }
        };

        let mut new_allocator = AtlasAllocator::new(new_size);
        let mut new_image = Image::new(new_size.cast());

        let mut new_mapping = BTreeMap::new();
        for (glyph_id, maybe_info) in self.mapping.iter().map(|(k, v)| (*k, *v)) {
            let info = match maybe_info {
                Some(info) => info,
                None => {
                    new_mapping.insert(glyph_id, None);
                    continue;
                }
            };
            let old_rect = info.location;
            let alloc = new_allocator
                .allocate(old_rect.size.cast())
                .ok_or(new_size)?;
            assert!(
                (old_rect.width() <= alloc.rectangle.width() as u32)
                    && (old_rect.height() <= alloc.rectangle.height() as u32),
            );
            let new_rect = Rect {
                origin: alloc.rectangle.min.cast::<u32>(),
                ..old_rect
            };
            self.image
                .with(|src| new_image.slice_mut(new_rect).copy_from(src.slice(old_rect)));

            new_mapping.insert(
                glyph_id,
                Some(GlyphImageInfo {
                    _alloc_id: alloc.id,
                    location: new_rect,
                    ..info
                }),
            );
        }
        self.mapping = new_mapping;
        self.allocator = new_allocator;
        self.image.resize(new_size.cast());
        self.image.update(|mut dst| dst.copy_from(new_image));
        Ok(())
    }

    fn alloc_space(&mut self, width: u32, height: u32) -> Option<Allocation> {
        if width == 0 || height == 0 {
            return None;
        }
        loop {
            match self
                .allocator
                .allocate((width as i32, height as i32).into())
            {
                Some(alloc) => break Some(alloc),
                None => {
                    let mut prev_size = None;
                    while let Err(size) = self.grow(prev_size) {
                        prev_size = Some(size);
                    }
                }
            }
        }
    }

    pub fn add_glyph(&mut self, scaler: &mut Scaler<'_>, glyph_id: GlyphId) {
        if self.mapping.contains_key(&glyph_id) {
            return;
        }

        let (image, placement) = match self.render.render(scaler, glyph_id) {
            Some(img) => (
                Image::with_data((img.placement.width, img.placement.height), img.data),
                img.placement,
            ),
            None => {
                assert!(self.mapping.insert(glyph_id, None).is_none());
                return;
            }
        };
        let info = match self.alloc_space(placement.width, placement.height) {
            Some(alloc) => {
                let rect = Rect {
                    origin: alloc.rectangle.min.cast(),
                    size: Size2D::new(placement.width, placement.height),
                };
                self.image
                    .update_part(|mut dst| dst.copy_from(&image), rect);
                Some(GlyphImageInfo {
                    _alloc_id: alloc.id,
                    location: rect,
                    placement,
                })
            }
            None => None,
        };
        assert!(self.mapping.insert(glyph_id, info).is_none());
    }

    pub fn glyph_info(&self, glyph_id: GlyphId) -> Option<GlyphImageInfo> {
        self.mapping.get(&glyph_id).and_then(|x| *x)
    }

    pub fn image(&self) -> &AtlasImage<u8> {
        &self.image
    }

    pub fn debug_svg(&self) -> Vec<u8> {
        let mut svg_data = Vec::new();
        self.allocator.dump_svg(&mut svg_data).unwrap();
        svg_data
    }
}
