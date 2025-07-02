use std::rc::Rc;

use anyhow::{Context, Result};
use glam::Mat4;

use crate::{State, object::Object};

pub struct Frame<'a> {
    state: Rc<State<'a>>,
    surface: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
}

impl<'a> Frame<'a> {
    pub fn new(state: Rc<State<'a>>) -> Result<Self> {
        let surface = state
            .surface
            .get_current_texture()
            .context("Failed to acquire next swap chain texture")?;
        let view = surface
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        Ok(Frame {
            state,
            surface,
            view,
        })
    }

    pub fn present(self) {
        self.surface.present()
    }

    pub fn render<T: Object>(&self, object: &T) {
        let vertices = object.vertices();

        let aspect_ratio = {
            let (width, height) = self.state.size();
            width as f32 / height as f32
        };
        let transformation =
            Mat4::orthographic_rh(-aspect_ratio, aspect_ratio, -1.0, 1.0, -1.0, 1.0);
        let bind_group = object.create_uniforms(transformation);

        let mut encoder = self
            .state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            {
                renderpass.push_debug_group("prepare");
                renderpass.set_pipeline(object.pipeline());
                renderpass.set_bind_group(0, &bind_group, &[]);
                renderpass.set_vertex_buffer(0, vertices.vertex_buffer.slice(..));
                renderpass
                    .set_index_buffer(vertices.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                renderpass.pop_debug_group();
            }
            renderpass.insert_debug_marker("draw");
            renderpass.draw_indexed(0..vertices.count, 0, 0..1);
        }

        self.state.queue.submit(Some(encoder.finish()));
    }
}
