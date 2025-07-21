use alloc::vec::Vec;

use anyhow::Result;
use wgpu::util::DeviceExt;

use wgame_gfx::Renderer;

#[derive(Default)]
pub struct TextStorage {
    pub count: u32,
    pub data: Vec<u8>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextRenderer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    device: wgpu::Device,
}

impl Renderer for TextRenderer {
    type Storage = TextStorage;

    fn new_storage(&self) -> Self::Storage {
        TextStorage::default()
    }
    fn draw(&self, instances: &Self::Storage, pass: &mut wgpu::RenderPass) -> Result<()> {
        let instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instances"),
                contents: &instances.data,
                usage: wgpu::BufferUsages::VERTEX,
            });

        {
            pass.push_debug_group("prepare");
            pass.set_pipeline(&self.pipeline);
            for (i, bind_group) in self.uniforms.iter().enumerate() {
                pass.set_bind_group(i as u32, bind_group, &[]);
            }
            pass.set_vertex_buffer(0, self.vertices.vertex_buffer.slice(..));
            if let Some(index_buffer) = &self.vertices.index_buffer {
                pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            }
            pass.set_vertex_buffer(1, instance_buffer.slice(..));
            pass.pop_debug_group();
        }

        pass.insert_debug_marker("draw");
        if self.vertices.index_buffer.is_some() {
            pass.draw_indexed(0..self.vertices.count, 0, 0..instances.count);
        } else {
            pass.draw(0..self.vertices.count, 0..instances.count);
        }

        Ok(())
    }
}
