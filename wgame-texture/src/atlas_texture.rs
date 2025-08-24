use core::marker::PhantomData;

use glam::Affine2;
use half::f16;
use rgb::Rgba;

use crate::{
    SharedState,
    types::{Rect, Texel},
};

#[derive(Clone)]
pub struct AtlasTexture<T: Texel> {
    state: SharedState,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    _ghost: PhantomData<T>,
}

impl<T: Texel> AtlasTexture<T> {
    const INITIAL_SIZE: (u32, u32) = (16, 16);

    pub(crate) fn new(state: &SharedState, format: wgpu::TextureFormat) -> Self {
        let device = state.device();
        assert!(T::is_format_supported(format));

        let size = Self::INITIAL_SIZE;
        let extent = wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = match format.sample_type(None, None) {
            Some(wgpu::TextureSampleType::Uint) => {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &state.uint_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    }],
                    label: None,
                })
            }
            Some(wgpu::TextureSampleType::Float { filterable: true }) => {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &state.float_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&state.float_sampler),
                        },
                    ],
                    label: None,
                })
            }
            _ => panic!("Unsupported texture format: {format:?}"),
        };

        Self {
            state: state.clone(),
            texture,
            bind_group,
            _ghost: PhantomData,
        }
    }

    fn write(&self, image: &GrayImage, bbox: Box2D<u32>) {
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

    pub fn write(&self, rect: Rect, data: &[Rgba<f16>]) {
        let format = self.texture.format();
        let bytes_per_block = format.block_copy_size(None).unwrap();
        assert_eq!(
            (self.extent.width * self.extent.height * bytes_per_block) as usize,
            size_of_val(data),
            "Texture data size mismatch"
        );
        self.state.queue().write_texture(
            self.texture.as_image_copy(),
            bytemuck::cast_slice(data),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(
                    (self.extent.width / format.block_dimensions().0) * bytes_per_block,
                ),
                rows_per_image: Some(self.extent.height / format.block_dimensions().1),
            },
            self.extent,
        );
    }

    pub fn bind_group_layout(&self) -> wgpu::BindGroupLayout {
        self.state.float_bind_group_layout.clone()
    }
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
