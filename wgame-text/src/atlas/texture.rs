use etagere::euclid::default::Box2D;
use image::GrayImage;
use wgpu::Extent3d;

use wgame_gfx::Graphics;

use super::mapping::Atlas;

struct Texture {
    size: Extent3d,
    inner: wgpu::Texture,
    view: wgpu::TextureView,
}

impl Texture {
    pub fn new(state: &Graphics, size: (u32, u32)) -> Self {
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

    pub fn size(&self) -> (u32, u32) {
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

    pub fn sync(this: &mut Option<Self>, state: &Graphics, atlas: &mut Atlas) {
        let texture_size = match this {
            Some(texture) => texture.size(),
            None => (0, 0),
        };
        let image_size = {
            let image = atlas.image();
            (image.width(), image.height())
        };
        if texture_size != image_size {
            *this = None;
        }

        let mut total_bbox = None;
        let texture = match &mut *this {
            out @ None => {
                total_bbox = Some(Box2D {
                    min: (0, 0).into(),
                    max: image_size.into(),
                });
                out.insert(Self::new(state, image_size))
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
            texture.write(state, atlas.image(), bbox);
        }
    }
}
