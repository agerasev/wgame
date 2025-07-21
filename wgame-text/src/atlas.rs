use core::cell::RefCell;

use alloc::{rc::Rc, vec::Vec};

use etagere::{AllocId, Allocation, AtlasAllocator};
use hashbrown::HashMap;
use image::{GenericImage, GenericImageView, GrayImage, math::Rect};
use swash::{
    GlyphId,
    scale::{Render, Scaler, Source, StrikeWith},
    zeno::Placement,
};
use wgame_gfx::Graphics;

use crate::{ContextKey, Font};

struct GlyphImageInfo {
    alloc_id: AllocId,
    location: Rect,
    placement: Placement,
}

struct Atlas {
    allocator: AtlasAllocator,
    map: HashMap<GlyphId, Option<GlyphImageInfo>>,
    image: GrayImage,
    render: Render<'static>,
}

#[derive(Clone)]
pub struct FontAtlas {
    pub(crate) state: Graphics,
    font: Font,
    size: f32,
    atlas: Rc<RefCell<Atlas>>,
}

impl FontAtlas {
    pub fn new(state: &Graphics, font: &Font, size: f32) -> Self {
        let init_dim = ((4.0 * size).ceil().clamp(u32::MIN as f32, i32::MAX as f32) as u32)
            .next_power_of_two();
        Self {
            state: state.clone(),
            font: font.clone(),
            size,
            atlas: Rc::new(RefCell::new(Atlas::new(init_dim))),
        }
    }

    pub fn add_chars(&self, codepoints: impl IntoIterator<Item = impl Into<u32>>) {
        let font_ref = self.font.as_ref();
        self.add_glyphs(codepoints.into_iter().map(|c| font_ref.charmap().map(c)));
    }
    pub(crate) fn add_glyphs(&self, glyphs: impl IntoIterator<Item = GlyphId>) {
        let reg = self.state.registry().get_or_init(ContextKey);

        let mut context = reg.scale.borrow_mut();
        let mut scaler = context
            .builder(self.font.as_ref())
            .size(self.size)
            .hint(false)
            .build();

        for glyph_id in glyphs {
            self.atlas.borrow_mut().add_glyph(&mut scaler, glyph_id);
        }
    }

    pub fn font(&self) -> &Font {
        &self.font
    }
    pub fn size(&self) -> f32 {
        self.size
    }
    pub fn image(&self) -> GrayImage {
        self.atlas.borrow().image.clone()
    }
    pub fn debug_svg(&self) -> Vec<u8> {
        let mut svg_data = Vec::new();
        self.atlas
            .borrow()
            .allocator
            .dump_svg(&mut svg_data)
            .unwrap();
        svg_data
    }
}

impl Atlas {
    fn new(init_dim: u32) -> Self {
        Self {
            allocator: AtlasAllocator::new((init_dim as i32, init_dim as i32).into()),
            map: HashMap::default(),
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
        if self.map.contains_key(&glyph_id) {
            return;
        }

        let (image, placement) = match self.render.render(scaler, glyph_id) {
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
}
