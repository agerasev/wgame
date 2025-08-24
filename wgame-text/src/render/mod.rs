mod library;
mod texture;

use anyhow::Result;
use glam::{Mat4, Vec4};
use wgpu::util::DeviceExt;

use wgame_gfx::{Renderer, Resources, utils::AnyOrder};

pub use self::{library::TextLibrary, texture::FontTexture};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextResources {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture: FontTexture,
    pipeline: wgpu::RenderPipeline,
    device: wgpu::Device,
}

impl TextResources {
    pub fn new(font: &FontTexture) -> Self {
        let library = &font.library;
        let pipeline = library.pipeline.clone();

        Self {
            vertex_buffer: library.vertex_buffer.clone(),
            index_buffer: library.index_buffer.clone(),
            pipeline,
            texture: font.clone(),
            device: library.device().clone(),
        }
    }
}

pub struct GlyphInstance {
    pub xform: Mat4,
    pub tex_coord: Vec4,
    pub color: Vec4,
}

#[derive(Default)]
pub struct TextStorage {
    pub(crate) instances: Vec<GlyphInstance>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextRenderer {
    resources: TextResources,
    bind_group: wgpu::BindGroup,
    instance_buffer: wgpu::Buffer,
    instance_count: u32,
}

impl Resources for TextResources {
    type Storage = TextStorage;
    type Renderer = TextRenderer;

    fn new_storage(&self) -> Self::Storage {
        TextStorage::default()
    }
    fn make_renderer(&self, storage: &Self::Storage) -> Result<Self::Renderer> {
        let mut bytes = Vec::new();
        for instance in &storage.instances {
            bytes.extend_from_slice(bytemuck::cast_slice(&[instance.xform]));
            bytes.extend_from_slice(bytemuck::cast_slice(&[instance.tex_coord]));
            bytes.extend_from_slice(bytemuck::cast_slice(&[instance.color]));
        }
        let instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instances"),
                contents: &bytes,
                usage: wgpu::BufferUsages::VERTEX,
            });

        let texture_view = self.texture.sync().unwrap();
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            }],
            label: None,
        });

        Ok(TextRenderer {
            resources: self.clone(),
            bind_group,
            instance_buffer,
            instance_count: storage.instances.len() as u32,
        })
    }
}

impl Renderer for TextRenderer {
    fn draw(&self, pass: &mut wgpu::RenderPass) -> Result<()> {
        pass.push_debug_group("prepare");
        pass.set_pipeline(&self.resources.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.resources.vertex_buffer.slice(..));
        pass.set_index_buffer(
            self.resources.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.pop_debug_group();

        pass.insert_debug_marker("draw");
        pass.draw_indexed(0..6, 0, 0..self.instance_count);

        Ok(())
    }
}
impl AnyOrder for TextRenderer {
    fn order(&self) -> i64 {
        // Text is rendered over other shapes by default
        1 << 16
    }
}
