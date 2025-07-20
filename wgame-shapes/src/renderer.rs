use alloc::vec::Vec;

use anyhow::Result;
use smallvec::SmallVec;

use wgame_gfx::Renderer;
use wgpu::util::DeviceExt;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct VertexBuffers {
    pub count: u32,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ShapeRenderer {
    pub order: i64,
    pub vertices: VertexBuffers,
    pub uniforms: SmallVec<[wgpu::BindGroup; 2]>,
    pub pipeline: wgpu::RenderPipeline,
    pub device: wgpu::Device,
}

#[derive(Default)]
pub struct InstancesStorage {
    pub count: u32,
    pub data: Vec<u8>,
}

impl Renderer for ShapeRenderer {
    type Storage = InstancesStorage;

    fn new_storage(&self) -> Self::Storage {
        Default::default()
    }

    fn draw(&self, instances: &Self::Storage, pass: &mut wgpu::RenderPass) -> Result<()> {
        log::trace!("Rendering {} instances", instances.count);

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
