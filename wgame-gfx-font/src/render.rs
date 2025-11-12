use anyhow::Result;
use glam::{Mat4, Vec4};
use wgame_font::swash::GlyphId;
use wgame_gfx::{Renderer, Resource, utils::AnyOrder};
use wgame_gfx_texture::TextureResource;
use wgame_shader::{Attribute, BytesSink};
use wgpu::util::DeviceExt;

use crate::FontTexture;

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

pub struct TextInstance {
    pub texture: FontTexture,
    pub glyphs: Vec<GlyphInstance>,
    pub color: Vec4,
}

pub struct GlyphInstance {
    pub xform: Mat4,
    pub id: GlyphId,
}

#[derive(Attribute)]
struct GlyphAttribute {
    xform: Mat4,
    tex_rect: Vec4,
    color: Vec4,
}

#[derive(Default)]
pub struct TextStorage {
    pub(crate) instances: Vec<TextInstance>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextRenderer {
    resource: TextResource,
    instance_buffer: wgpu::Buffer,
    instance_count: u32,
}

impl Resource for TextResource {
    type Storage = TextStorage;
    type Renderer = TextRenderer;

    fn new_storage(&self) -> Self::Storage {
        TextStorage::default()
    }
    fn make_renderer(&self, storage: &Self::Storage) -> Result<Self::Renderer> {
        let mut bytes = BytesSink::default();
        let mut instance_count = 0;
        for text in &storage.instances {
            for glyph in &text.glyphs {
                let rect = text.texture.glyph_rect(glyph.id).unwrap();
                let attr = GlyphAttribute {
                    xform: glyph.xform,
                    tex_rect: Vec4::new(
                        rect.origin.x as f32,
                        rect.origin.y as f32,
                        rect.size.width as f32,
                        rect.size.height as f32,
                    ),
                    color: text.color,
                };
                attr.store(&mut bytes);
                instance_count += 1;
            }
        }
        let instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instances"),
                contents: bytes.data(),
                usage: wgpu::BufferUsages::VERTEX,
            });

        Ok(TextRenderer {
            resource: self.clone(),
            instance_buffer,
            instance_count,
        })
    }
}

impl Renderer for TextRenderer {
    fn draw(&self, pass: &mut wgpu::RenderPass) -> Result<()> {
        pass.push_debug_group("prepare");
        pass.set_pipeline(&self.resource.pipeline);
        pass.set_bind_group(0, &self.resource.texture.bind_group(), &[]);
        pass.set_vertex_buffer(0, self.resource.vertex_buffer.slice(..));
        pass.set_index_buffer(
            self.resource.index_buffer.slice(..),
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
