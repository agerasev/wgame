use anyhow::Result;
use glam::{Mat4, Vec4};
use swash::GlyphId;
use wgame_gfx::{Renderer, Resources, utils::AnyOrder};
use wgame_texture::TextureResources;
use wgpu::util::DeviceExt;

use crate::FontTexture;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextResources {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture: TextureResources<u8>,
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
            texture: FontTexture::texture(font).resources(),
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

#[derive(Default)]
pub struct TextStorage {
    pub(crate) instances: Vec<TextInstance>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextRenderer {
    resources: TextResources,
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
        for text in &storage.instances {
            for glyph in &text.glyphs {
                let rect = text.texture.glyph_rect(glyph.id).unwrap();
                bytes.extend_from_slice(bytemuck::cast_slice(&[glyph.xform]));
                bytes.extend_from_slice(bytemuck::cast_slice(&[
                    rect.origin.x as f32,
                    rect.origin.y as f32,
                    rect.size.width as f32,
                    rect.size.height as f32,
                ]));
                bytes.extend_from_slice(bytemuck::cast_slice(&[text.color]));
            }
        }
        let instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instances"),
                contents: &bytes,
                usage: wgpu::BufferUsages::VERTEX,
            });

        Ok(TextRenderer {
            resources: self.clone(),
            instance_buffer,
            instance_count: storage.instances.len() as u32,
        })
    }
}

impl Renderer for TextRenderer {
    fn draw(&self, pass: &mut wgpu::RenderPass) -> Result<()> {
        pass.push_debug_group("prepare");
        pass.set_pipeline(&self.resources.pipeline);
        pass.set_bind_group(0, &self.resources.texture.bind_group(), &[]);
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
