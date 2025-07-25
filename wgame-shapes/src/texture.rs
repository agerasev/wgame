use glam::{Affine2, Vec2};
use half::f16;
use rgb::Rgba;

use wgame_gfx::types::Color;

use crate::{Graphics, Library, LibraryState};

#[derive(Clone)]
pub struct Texture {
    state: LibraryState,
    extent: wgpu::Extent3d,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    xform: Affine2,
}

impl Texture {
    pub(crate) fn create_bind_group_layout(state: &Graphics) -> wgpu::BindGroupLayout {
        state
            .device()
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            })
    }
    pub(crate) fn create_sampler(state: &Graphics) -> wgpu::Sampler {
        state.device().create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    }

    pub(crate) fn new(state: &LibraryState, size: (u32, u32)) -> Self {
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
            layout: &state.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&state.texture_sampler),
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
        self.state.inner.queue().write_texture(
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
        self.state.texture_bind_group_layout.clone()
    }
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn coord_xform(&self) -> Affine2 {
        self.xform
    }
}

impl Library {
    pub fn texture(&self, size: impl Into<(u32, u32)>) -> Texture {
        Texture::new(self, size.into())
    }
    pub fn texture_with_data(
        &self,
        size: impl Into<(u32, u32)>,
        data: impl AsRef<[Rgba<f16>]>,
    ) -> Texture {
        let tex = self.texture(size);
        tex.write(data.as_ref());
        tex
    }

    pub fn gradient<T: Color, const N: usize>(&self, colors: [T; N]) -> Texture {
        self.gradient2([colors])
    }

    pub fn gradient2<T: Color, const M: usize, const N: usize>(
        &self,
        colors: [[T; M]; N],
    ) -> Texture {
        let colors = colors.map(|row| row.map(|color| color.to_rgba()));
        let pix_size = Vec2::new(M as f32, N as f32).recip();
        self.texture_with_data((M as u32, N as u32), colors.as_flattened())
            .transform_coord(Affine2::from_scale_angle_translation(
                1.0 - pix_size,
                0.0,
                0.5 * pix_size,
            ))
    }
}
