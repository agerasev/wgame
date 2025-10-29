use alloc::rc::Rc;
use core::cell::RefCell;

use glam::Affine2;
use half::f16;
use rgb::Rgba;
use wgame_img::{Image, Pixel, Rect};

use crate::SharedState;

pub trait TextureData {
    type Pixel: Pixel;
    fn image(&self) -> Image<Self::Pixel>;
    fn take_update(&mut self) -> Option<Rect>;
}

#[derive(Clone)]
pub struct InnerTexture<T: TextureData> {
    data: T,
    state: SharedState,
    extent: wgpu::Extent3d,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    xform: Affine2,
}

#[derive(Clone)]
pub struct Texture<T: TextureData> {
    inner: Rc<RefCell<InnerTexture<T>>>,
    xform: Affine2,
}

impl Texture {
    pub(crate) fn new(state: &SharedState, size: (u32, u32)) -> Self {
        let device = state.device();

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
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
        });

        Self {
            state: state.clone(),
            extent,
            texture,
            bind_group,
            xform: Affine2::IDENTITY,
        }
    }

    pub fn transform_coord(self, xform: Affine2) -> Self {
        Self {
            xform: xform * self.xform,
            ..self
        }
    }

    pub fn write(&self, data: &[Rgba<f16>]) {
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

    pub fn coord_xform(&self) -> Affine2 {
        self.xform
    }
}
