use core::cell::RefCell;

use crate::Texture;

/// Shared registry
pub struct Registry {
    pub instance_buffer: RefCell<Option<wgpu::Buffer>>,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub texture_sampler_linear: wgpu::Sampler,
}

impl Registry {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            instance_buffer: Default::default(),
            texture_bind_group_layout: Texture::create_bind_group_layout(device),
            texture_sampler_linear: Texture::create_sampler_linear(device),
        }
    }
}
