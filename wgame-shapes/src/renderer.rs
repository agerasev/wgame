use alloc::vec::Vec;

use anyhow::Result;

use wgame_gfx::{Renderer, Resources, utils::AnyOrder};
use wgame_texture::TextureResources;
use wgpu::util::DeviceExt;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct VertexBuffers {
    pub count: u32,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ShapeResources {
    pub order: i64,
    pub vertices: VertexBuffers,
    pub texture: TextureResources,
    pub uniforms: Option<wgpu::BindGroup>,
    pub pipeline: wgpu::RenderPipeline,
    pub device: wgpu::Device,
}

#[derive(Default)]
pub struct VertexStorage {
    pub count: u32,
    pub data: Vec<u8>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ShapeRenderer {
    pub resources: ShapeResources,
    pub instance_buffer: wgpu::Buffer,
    pub instance_count: u32,
}
impl AnyOrder for ShapeRenderer {}

impl Resources for ShapeResources {
    type Storage = VertexStorage;
    type Renderer = ShapeRenderer;

    fn new_storage(&self) -> Self::Storage {
        Default::default()
    }

    fn make_renderer(&self, instances: &Self::Storage) -> Result<Self::Renderer> {
        let instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instances"),
                contents: &instances.data,
                usage: wgpu::BufferUsages::VERTEX,
            });

        Ok(ShapeRenderer {
            resources: self.clone(),
            instance_buffer,
            instance_count: instances.count,
        })
    }
}

impl ShapeResources {
    fn uniforms(&self) -> impl IntoIterator<Item = wgpu::BindGroup> {
        [self.texture.bind_group().clone()]
            .into_iter()
            .chain(self.uniforms.clone())
    }
}

impl Renderer for ShapeRenderer {
    fn draw(&self, pass: &mut wgpu::RenderPass<'_>) -> Result<()> {
        log::trace!("Rendering {} instances", self.instance_count);

        pass.push_debug_group("prepare");
        pass.set_pipeline(&self.resources.pipeline);
        for (i, bind_group) in self.resources.uniforms().into_iter().enumerate() {
            pass.set_bind_group(i as u32, &bind_group, &[]);
        }
        pass.set_vertex_buffer(0, self.resources.vertices.vertex_buffer.slice(..));
        if let Some(index_buffer) = &self.resources.vertices.index_buffer {
            pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        }
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.pop_debug_group();

        pass.insert_debug_marker("draw");
        if self.resources.vertices.index_buffer.is_some() {
            pass.draw_indexed(0..self.resources.vertices.count, 0, 0..self.instance_count);
        } else {
            pass.draw(0..self.resources.vertices.count, 0..self.instance_count);
        }

        Ok(())
    }
}
