use std::marker::PhantomData;

use derivative::Derivative;
use smallvec::SmallVec;
use wgame_gfx::{Camera, Context, Renderer, Resource, Storage};
use wgame_gfx_texture::TextureResource;
use wgame_shader::{Attribute, BytesSink};
use wgpu::util::DeviceExt;

use crate::{Mesh, shader::InstanceData};

#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    Hash(bound = ""),
    Debug(bound = "")
)]
pub struct ShapeResource<T: Attribute> {
    pub vertices: Mesh,
    pub texture: TextureResource,
    pub uniforms: Option<wgpu::BindGroup>,
    pub pipeline: wgpu::RenderPipeline,
    pub device: wgpu::Device,
    pub _ghost: PhantomData<T>,
}

pub struct ShapeStorage<T: Attribute> {
    resource: ShapeResource<T>,
    pub instances: Vec<InstanceData<T>>,
}

pub struct ShapeRenderer {
    geometry: Mesh,
    instance_count: u32,
    instance_buffer: wgpu::Buffer,
    uniforms: SmallVec<[wgpu::BindGroup; 2]>,
    pipeline: wgpu::RenderPipeline,
}

impl<T: Attribute> Resource for ShapeResource<T> {}

impl<T: Attribute> ShapeStorage<T> {
    pub(crate) fn new(resource: ShapeResource<T>) -> Self {
        Self {
            resource,
            instances: Vec::new(),
        }
    }
}

impl<T: Attribute> Storage for ShapeStorage<T> {
    type Context = Camera;
    type Resource = ShapeResource<T>;
    type Renderer = ShapeRenderer;

    fn resource(&self) -> Self::Resource {
        self.resource.clone()
    }
    fn bake(&self) -> Self::Renderer {
        let instance_count = self.instances.len() as u32;
        let mut buffer = BytesSink::default();
        for instance in &self.instances {
            instance.store(&mut buffer);
        }
        let buffer_data = buffer.into_data();
        let instance_buffer =
            self.resource
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("instances"),
                    contents: &buffer_data,
                    usage: wgpu::BufferUsages::VERTEX,
                });
        ShapeRenderer {
            geometry: self.resource.vertices.clone(),
            instance_count,
            instance_buffer,
            uniforms: self.resource.uniforms().into_iter().collect(),
            pipeline: self.resource.pipeline.clone(),
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

impl Renderer<Camera> for ShapeRenderer {
    fn render(&self, ctx: &Camera, pass: &mut wgpu::RenderPass<'_>) {
        pass.push_debug_group("prepare");
        pass.set_pipeline(&self.pipeline);
        for (i, bind_group) in [ctx.bind_group()].iter().chain(&self.uniforms).enumerate() {
            pass.set_bind_group(i as u32, bind_group, &[]);
        }
        pass.set_vertex_buffer(0, self.geometry.vertices().slice(..));
        if let Some(index_buffer) = self.geometry.indices() {
            pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        }
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.pop_debug_group();

        pass.insert_debug_marker("draw");
        if self.geometry.indices().is_some() {
            pass.draw_indexed(0..self.geometry.count(), 0, 0..self.instance_count);
        } else {
            pass.draw(0..self.geometry.count(), 0..self.instance_count);
        }
    }
}
