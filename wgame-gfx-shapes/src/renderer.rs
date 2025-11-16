use std::marker::PhantomData;

use derivative::Derivative;
use wgame_gfx::{Resource, utils::Order};
use wgame_gfx_texture::TextureResource;
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
pub struct ShapeResource<T: Attribute> {
    pub order: i64,
    pub vertices: VertexBuffers,
    pub texture: TextureResource,
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

impl<T: Attribute> Order for ShapeResource<T> {}

impl<T: Attribute> Resource for ShapeResource<T> {
    type Storage = InstanceStorage<T>;

    fn new_storage(&self) -> Self::Storage {
        Default::default()
    }

    fn render(&self, storage: &Self::Storage, pass: &mut wgpu::RenderPass) {
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

        log::trace!("Rendering {} instances", instance_count);

        pass.push_debug_group("prepare");
        pass.set_pipeline(&self.pipeline);
        for (i, bind_group) in self.uniforms().into_iter().enumerate() {
            pass.set_bind_group(i as u32, &bind_group, &[]);
        }
        pass.set_vertex_buffer(0, self.vertices.vertex_buffer.slice(..));
        if let Some(index_buffer) = &self.vertices.index_buffer {
            pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        }
        pass.set_vertex_buffer(1, instance_buffer.slice(..));
        pass.pop_debug_group();

        pass.insert_debug_marker("draw");
        if self.vertices.index_buffer.is_some() {
            pass.draw_indexed(0..self.vertices.count, 0, 0..instance_count);
        } else {
            pass.draw(0..self.vertices.count, 0..instance_count);
        }
    }
}

impl<T: Attribute> ShapeResource<T> {
    fn uniforms(&self) -> impl IntoIterator<Item = wgpu::BindGroup> {
        [self.texture.bind_group().clone()]
            .into_iter()
            .chain(self.uniforms.clone())
    }
}
