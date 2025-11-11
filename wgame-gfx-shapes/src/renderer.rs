use std::marker::PhantomData;

use anyhow::Result;
use derivative::Derivative;
use wgame_gfx::{Renderer, Resources, utils::AnyOrder};
use wgame_gfx_texture::TextureResources;
use wgame_shader::{Attribute, BytesSink};
use wgpu::util::DeviceExt;

use crate::primitive::InstanceData;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct VertexBuffers {
    pub count: u32,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
}

#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = ""),
    Hash(bound = ""),
    Debug(bound = "")
)]
pub struct ShapeResources<T: Attribute> {
    pub order: i64,
    pub vertices: VertexBuffers,
    pub texture: TextureResources,
    pub uniforms: Option<wgpu::BindGroup>,
    pub pipeline: wgpu::RenderPipeline,
    pub device: wgpu::Device,
    pub _ghost: PhantomData<T>,
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct InstanceStorage<T: Attribute> {
    pub instances: Vec<InstanceData<T>>,
}

#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = ""),
    Hash(bound = ""),
    Debug(bound = "")
)]
pub struct ShapeRenderer<T: Attribute> {
    pub resources: ShapeResources<T>,
    pub instance_buffer: wgpu::Buffer,
    pub instance_count: u32,
}
impl<T: Attribute> AnyOrder for ShapeRenderer<T> {}

impl<T: Attribute> Resources for ShapeResources<T> {
    type Storage = InstanceStorage<T>;
    type Renderer = ShapeRenderer<T>;

    fn new_storage(&self) -> Self::Storage {
        Default::default()
    }

    fn make_renderer(&self, storage: &Self::Storage) -> Result<Self::Renderer> {
        let instance_count = storage.instances.len() as u32;
        let mut buffer = BytesSink::default();
        for instance in &storage.instances {
            instance.store(&mut buffer);
        }
        let buffer_data = buffer.into_data();
        let instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instances"),
                contents: &buffer_data,
                usage: wgpu::BufferUsages::VERTEX,
            });

        Ok(ShapeRenderer {
            resources: self.clone(),
            instance_buffer,
            instance_count,
        })
    }
}

impl<T: Attribute> ShapeResources<T> {
    fn uniforms(&self) -> impl IntoIterator<Item = wgpu::BindGroup> {
        [self.texture.bind_group().clone()]
            .into_iter()
            .chain(self.uniforms.clone())
    }
}

impl<T: Attribute> Renderer for ShapeRenderer<T> {
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
