use crate::Camera;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Graphics {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: wgpu::TextureFormat,

    camera_bind_group_layout: wgpu::BindGroupLayout,
}

impl Graphics {
    pub(crate) fn new(
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            camera_bind_group_layout: Camera::create_bind_group_layout(&device),

            adapter,
            device,
            queue,
            format,
        }
    }

    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn camera_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bind_group_layout
    }
}
