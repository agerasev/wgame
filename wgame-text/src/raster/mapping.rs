use etagere::{AllocId, Allocation, AtlasAllocator};
use image::{GenericImage, GenericImageView, GrayImage, math::Rect};
use std::collections::BTreeMap;
use swash::{
    GlyphId,
    scale::{Render, Scaler, Source, StrikeWith},
    zeno::Placement,
};

#[derive(Clone, Copy, Debug)]
pub struct GlyphImageInfo {
    pub _alloc_id: AllocId,
    pub location: Rect,
    pub placement: Placement,
    pub texture_synced: bool,
}

pub struct FontAtlas {
    allocator: AtlasAllocator,
    mapping: BTreeMap<GlyphId, Option<GlyphImageInfo>>,
    image: GrayImage,
    render: Render<'static>,
}

impl FontAtlas {
    pub fn new(init_dim: u32) -> Self {
        Self {
            allocator: AtlasAllocator::new((init_dim as i32, init_dim as i32).into()),
            mapping: BTreeMap::default(),
            image: GrayImage::new(init_dim, init_dim),
            render: Render::new(&[
                Source::ColorOutline(0),
                Source::ColorBitmap(StrikeWith::BestFit),
                Source::Outline,
            ]),
        }
    }

    fn grow(&mut self, prev_size: Option<(i32, i32)>) -> Result<(), (i32, i32)> {
        let new_size = {
            let (width, height) = prev_size.unwrap_or_else(|| self.allocator.size().into());
            if width < height {
                (width * 2, height)
            } else {
                (width, height * 2)
            }
        };

        let mut new_allocator = AtlasAllocator::new(new_size.into());
        let mut new_image = GrayImage::new(new_size.0 as u32, new_size.1 as u32);

        let mut new_mapping = BTreeMap::new();
        for (glyph_id, maybe_info) in self.mapping.iter().map(|(k, v)| (*k, *v)) {
            let info = match maybe_info {
                Some(info) => info,
                None => {
                    new_mapping.insert(glyph_id, None);
                    continue;
                }
            };
            let rect = info.location;
            let alloc = new_allocator
                .allocate((rect.width as i32, rect.height as i32).into())
                .ok_or(new_size)?;
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

            new_mapping.insert(
                glyph_id,
                Some(GlyphImageInfo {
                    _alloc_id: alloc.id,
                    location: new_rect,
                    texture_synced: false,
                    ..info
                }),
            );
        }
        self.mapping = new_mapping;
        self.allocator = new_allocator;
        self.image = new_image;
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
                GrayImage::from_vec(img.placement.width, img.placement.height, img.data)
                    .expect("Cannot convert glyph image"),
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
                    x: alloc.rectangle.min.x as u32,
                    y: alloc.rectangle.min.y as u32,
                    width: placement.width,
                    height: placement.height,
                };
                self.image
                    .copy_from(&image, rect.x, rect.y)
                    .expect("Error copying glyph image");
                Some(GlyphImageInfo {
                    _alloc_id: alloc.id,
                    location: rect,
                    placement,
                    texture_synced: false,
                })
            }
            None => None,
        };
        assert!(self.mapping.insert(glyph_id, info).is_none());
    }

    pub fn get_glyph(&self, glyph_id: GlyphId) -> Option<GlyphImageInfo> {
        self.mapping.get(&glyph_id).and_then(|x| *x)
    }

    pub fn sync_glyphs(&mut self) -> impl Iterator<Item = (&GlyphId, &GlyphImageInfo)> {
        self.mapping
            .iter()
            .filter_map(|(k, v)| v.as_ref().map(|v| (k, v)))
            .filter(|(_k, v)| !v.texture_synced)
    }

    pub fn image(&self) -> &GrayImage {
        &self.image
    }

    pub fn debug_svg(&self) -> Vec<u8> {
        let mut svg_data = Vec::new();
        self.allocator.dump_svg(&mut svg_data).unwrap();
        svg_data
    }
}
