use std::ops::Deref;

use wgame_gfx::Graphics;

/// Shared state
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
