use etagere::{AllocId, Allocation, AtlasAllocator};
use hashbrown::HashMap;
use image::{GenericImage, GenericImageView, GrayImage, math::Rect};
use swash::{
    GlyphId,
    scale::{Render, Scaler, Source, StrikeWith},
    zeno::Placement,
};

pub struct GlyphImageInfo {
    pub alloc_id: AllocId,
    pub location: Rect,
    pub placement: Placement,
    pub texture_synced: bool,
}

pub struct Atlas {
    allocator: AtlasAllocator,
    mapping: HashMap<GlyphId, Option<GlyphImageInfo>>,
    image: GrayImage,
    render: Render<'static>,
}

impl Atlas {
    pub fn new(init_dim: u32) -> Self {
        Self {
            allocator: AtlasAllocator::new((init_dim as i32, init_dim as i32).into()),
            mapping: HashMap::default(),
            image: GrayImage::new(init_dim, init_dim),
            render: Render::new(&[
                Source::ColorOutline(0),
                Source::ColorBitmap(StrikeWith::BestFit),
                Source::Outline,
            ]),
        }
    }

    fn grow(&mut self) {
        let mut new_atlas = AtlasAllocator::new(self.allocator.size() * 2);
        let mut new_image = GrayImage::new(2 * self.image.width(), 2 * self.image.height());

        for maybe_info in self.mapping.values_mut() {
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
            info.texture_synced = false;
        }

        self.allocator = new_atlas;
        self.image = new_image;
    }

    fn alloc_space(&mut self, width: u32, height: u32) -> Allocation {
        loop {
            match self
                .allocator
                .allocate((width as i32, height as i32).into())
            {
                Some(alloc) => break alloc,
                None => self.grow(),
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
            self.mapping
                .insert(
                    glyph_id,
                    Some(GlyphImageInfo {
                        alloc_id: alloc.id,
                        location: rect,
                        placement,
                        texture_synced: false
                    })
                )
                .is_none()
        );
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
