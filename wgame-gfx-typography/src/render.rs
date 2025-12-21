use glam::{Mat4, Vec4};
use wgame_gfx::{Camera, Renderer, Resource, Storage, types::Color};
use wgame_gfx_texture::TextureResource;
use wgame_shader::{Attribute, BytesSink};
use wgpu::util::DeviceExt;

use crate::{FontTexture, text::TextInstance};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextResource {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture: TextureResource<u8>,
    pipeline: wgpu::RenderPipeline,
    device: wgpu::Device,
}

impl TextResource {
    pub fn new(font: &FontTexture) -> Self {
        let library = &font.library;
        let pipeline = library.pipeline.clone();

        Self {
            vertex_buffer: library.vertex_buffer.clone(),
            index_buffer: library.index_buffer.clone(),
            pipeline,
            texture: font.inner().resource(),
            device: library.device().clone(),
        }
    }
}

#[derive(Attribute)]
struct GlyphAttribute {
    xform: Mat4,
    tex_rect: Vec4,
    color: Vec4,
}

pub struct TextStorage {
    resource: TextResource,
    pub(crate) instances: Vec<TextInstance>,
}

impl TextStorage {
    pub(crate) fn new(resource: TextResource) -> Self {
        Self {
            resource,
            instances: Vec::new(),
        }
    }
}

pub struct TextRenderer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_count: u32,
    instance_buffer: wgpu::Buffer,
    texture_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl Resource for TextResource {}

impl Storage for TextStorage {
    type Context = Camera;
    type Resource = TextResource;
    type Renderer = TextRenderer;

    fn resource(&self) -> Self::Resource {
        self.resource.clone()
    }

    fn bake(&self) -> Self::Renderer {
        let mut bytes = BytesSink::default();
        let mut instance_count = 0;
        for text in &self.instances {
            for glyph in text.glyphs.iter() {
                let rect = text.texture.glyph_rect(glyph.id).unwrap();
                let attr = GlyphAttribute {
                    xform: glyph.xform,
                    tex_rect: Vec4::new(
                        rect.origin.x as f32,
                        rect.origin.y as f32,
                        rect.size.width as f32,
                        rect.size.height as f32,
                    ),
                    color: text.color.to_vec4(),
                };
                attr.store(&mut bytes);
                instance_count += 1;
            }
        }
        let instance_buffer =
            self.resource
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("instances"),
                    contents: bytes.data(),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        TextRenderer {
            vertex_buffer: self.resource.vertex_buffer.clone(),
            index_buffer: self.resource.index_buffer.clone(),
            instance_count,
            instance_buffer,
            texture_bind_group: self.resource.texture.bind_group(),
            pipeline: self.resource.pipeline.clone(),
        }
    }
}

impl Renderer<Camera> for TextRenderer {
    fn render(&self, ctx: &Camera, pass: &mut wgpu::RenderPass<'_>) {
        pass.push_debug_group("prepare");
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.texture_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.pop_debug_group();

        pass.insert_debug_marker("draw");
        pass.draw_indexed(0..6, 0, 0..self.instance_count);
    }
}
