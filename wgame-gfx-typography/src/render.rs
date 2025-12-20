use glam::{Mat4, Vec4};
use wgame_gfx::{Resource, types::Color};
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

#[derive(Default)]
pub struct TextStorage {
    pub(crate) instances: Vec<TextInstance>,
}

impl Resource for TextResource {
    type Storage = TextStorage;

    fn new_storage(&self) -> Self::Storage {
        TextStorage::default()
    }
    fn render(&self, storage: &Self::Storage, pass: &mut wgpu::RenderPass) {
        let mut bytes = BytesSink::default();
        let mut instance_count = 0;
        for text in &storage.instances {
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
        let instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instances"),
                contents: bytes.data(),
                usage: wgpu::BufferUsages::VERTEX,
            });

        pass.push_debug_group("prepare");
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.texture.bind_group(), &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        pass.set_vertex_buffer(1, instance_buffer.slice(..));
        pass.pop_debug_group();

        pass.insert_debug_marker("draw");
        pass.draw_indexed(0..6, 0, 0..instance_count);
    }
}
