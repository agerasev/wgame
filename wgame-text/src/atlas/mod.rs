mod mapping;

use core::cell::RefCell;
use std::rc::Rc;

use etagere::euclid::default::Box2D;
use image::GrayImage;
use swash::{GlyphId, scale::ScaleContext};
use wgpu::Extent3d;

use wgame_gfx::Graphics;

use self::mapping::Atlas;
use crate::Font;

thread_local! {
    static CONTEXT: RefCell<ScaleContext> = Default::default();
}

struct Texture {
    size: Extent3d,
    inner: wgpu::Texture,
    view: wgpu::TextureView,
}

#[derive(Clone)]
pub struct FontAtlas {
    state: Graphics,
    font: Font,
    size: f32,
    atlas: Rc<RefCell<Atlas>>,
    texture: Rc<RefCell<Option<Texture>>>,
}

impl Texture {
    fn new(state: &Graphics, size: (u32, u32)) -> Self {
        let device = state.device();

        let size = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            size,
            inner: texture,
            view,
        }
    }

    fn size(&self) -> (u32, u32) {
        (self.size.width, self.size.height)
    }

    fn write(&self, state: &Graphics, image: &GrayImage, bbox: Box2D<u32>) {
        let image_size = (image.width(), image.height());
        assert_eq!(
            self.size(),
            image_size,
            "Image and texture are of different size"
        );
        assert!(
            (bbox.min.x <= bbox.max.x && bbox.max.x <= image_size.0)
                && (bbox.min.y <= bbox.max.y && bbox.max.y <= image_size.1)
        );
        state.queue().write_texture(
            self.inner.as_image_copy(),
            bytemuck::cast_slice(image.as_raw()),
            wgpu::TexelCopyBufferLayout {
                offset: (bbox.min.y as u64 * image_size.0 as u64) + bbox.min.x as u64,
                bytes_per_row: Some(image_size.0),
                rows_per_image: Some(image_size.1),
            },
            Extent3d {
                width: bbox.width(),
                height: bbox.height(),
                depth_or_array_layers: 1,
            },
        );
    }
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
            texture: Default::default(),
        }
    }

    pub fn add_chars(&self, codepoints: impl IntoIterator<Item = impl Into<u32>>) {
        let font_ref = self.font.as_ref();
        self.add_glyphs(codepoints.into_iter().map(|c| font_ref.charmap().map(c)));
    }
    pub(crate) fn add_glyphs(&self, glyphs: impl IntoIterator<Item = GlyphId>) {
        let mut atlas = self.atlas.borrow_mut();

        CONTEXT.with_borrow_mut(|context| {
            let mut scaler = context
                .builder(self.font.as_ref())
                .size(self.size)
                .hint(false)
                .build();

            for glyph_id in glyphs {
                atlas.add_glyph(&mut scaler, glyph_id);
            }
        });

        let mut texture_opt = self.texture.borrow_mut();
        let texture_size = match &*texture_opt {
            Some(texture) => texture.size(),
            None => (0, 0),
        };
        let image_size = {
            let image = atlas.image();
            (image.width(), image.height())
        };
        if texture_size != image_size {
            *texture_opt = None;
        }

        let mut total_bbox = None;
        let texture = match &mut *texture_opt {
            out @ None => {
                total_bbox = Some(Box2D {
                    min: (0, 0).into(),
                    max: image_size.into(),
                });
                out.insert(Texture::new(&self.state, image_size))
            }
            Some(texture) => {
                for (_glyph_id, glyph_info) in atlas.sync_glyphs() {
                    let bbox = {
                        let rect = glyph_info.location;
                        Box2D {
                            min: (rect.x, rect.y).into(),
                            max: (rect.x + rect.width, rect.y + rect.height).into(),
                        }
                    };
                    total_bbox = Some(match total_bbox {
                        None => bbox,
                        Some(total) => total.union(&bbox),
                    });
                }
                texture
            }
        };
        if let Some(bbox) = total_bbox {
            texture.write(&self.state, atlas.image(), bbox);
        }
    }

    pub fn font(&self) -> &Font {
        &self.font
    }
    pub fn size(&self) -> f32 {
        self.size
    }
    pub fn image(&self) -> GrayImage {
        self.atlas.borrow().image().clone()
    }
    pub fn atlas_svg(&self) -> Vec<u8> {
        self.atlas.borrow().debug_svg()
    }
}
