use std::ops::Deref;

use wgame_gfx::Graphics;

/// Shared samplers and binding layouts.
///
/// Float sampling binds a filterable 2D texture at binding 0, sampler at 1, and a
/// 16-byte `vec4<u32>` uniform at 2. Its first component is 1 for premultiplied RGB,
/// 0 for straight RGB; the remaining components are reserved. Custom shaders
/// should divide sampled RGB by nonzero alpha when this flag is set, with zero
/// RGB for zero alpha. Built-in shapes and lighting perform this conversion.
/// Integer sampling uses only the texture at binding 0.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TexturingState {
    inner: Graphics,
    pub uint_bind_group_layout: wgpu::BindGroupLayout,
    pub float_bind_group_layout: wgpu::BindGroupLayout,
    pub nearest_sampler: wgpu::Sampler,
    pub linear_sampler: wgpu::Sampler,
}

impl Deref for TexturingState {
    type Target = Graphics;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TexturingState {
    pub fn new(state: &Graphics) -> Self {
        Self {
            inner: state.clone(),
            uint_bind_group_layout: create_uint_bind_group_layout(state),
            float_bind_group_layout: create_float_bind_group_layout(state),
            linear_sampler: create_sampler(state, wgpu::FilterMode::Linear),
            nearest_sampler: create_sampler(state, wgpu::FilterMode::Nearest),
        }
    }

    pub fn bind_group_layout(&self, format: wgpu::TextureFormat) -> wgpu::BindGroupLayout {
        match format.sample_type(None, None) {
            Some(wgpu::TextureSampleType::Uint) => self.uint_bind_group_layout.clone(),
            Some(wgpu::TextureSampleType::Float { filterable: true }) => {
                self.float_bind_group_layout.clone()
            }
            _ => panic!("Unsupported texture format: {format:?}"),
        }
    }
}

fn create_uint_bind_group_layout(state: &Graphics) -> wgpu::BindGroupLayout {
    state
        .device()
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("wgame_uint_texture_bind_group"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Uint,
                },
                count: None,
            }],
        })
}

fn create_float_bind_group_layout(state: &Graphics) -> wgpu::BindGroupLayout {
    state
        .device()
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("wgame_float_texture_bind_group"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZero::new(16),
                    },
                    count: None,
                },
            ],
        })
}

fn create_sampler(state: &Graphics, mag_filter: wgpu::FilterMode) -> wgpu::Sampler {
    state.device().create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}
