#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

mod atlas;
mod atlas_texture;
mod texture;
mod types;

use core::ops::Deref;

use wgame_gfx::Graphics;

pub use self::texture::Texture;

/// Shared state
#[derive(Clone)]
pub struct SharedState {
    inner: Graphics,
    uint_bind_group_layout: wgpu::BindGroupLayout,
    float_bind_group_layout: wgpu::BindGroupLayout,
    float_sampler: wgpu::Sampler,
}

impl Deref for SharedState {
    type Target = Graphics;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl SharedState {
    pub fn new(state: &Graphics) -> Self {
        Self {
            inner: state.clone(),
            uint_bind_group_layout: create_uint_bind_group_layout(state),
            float_bind_group_layout: create_float_bind_group_layout(state),
            float_sampler: create_float_sampler(state),
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

fn create_float_sampler(state: &Graphics) -> wgpu::Sampler {
    state.device().create_sampler(&wgpu::SamplerDescriptor {
        label: Some("wgame_float_sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}
