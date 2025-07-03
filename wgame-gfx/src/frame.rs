use std::rc::Rc;

use anyhow::{Context, Result};
use glam::Mat4;

use crate::{object::Object, state::State};

pub struct Frame<'a> {
    state: Rc<State<'a>>,
    surface: Option<wgpu::SurfaceTexture>,
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
            surface: Some(surface),
            view,
        })
    }
}

impl Drop for Frame<'_> {
    fn drop(&mut self) {
        self.surface
            .take()
            .expect("Inner frame is already taken")
            .present();
    }
}

impl Frame<'_> {
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
                    // depth_slice: None,
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
                renderpass.push_debug_group("Prepare data for draw.");
                renderpass.set_pipeline(object.pipeline());
                renderpass.set_bind_group(0, &bind_group, &[]);
                renderpass.set_vertex_buffer(0, vertices.buffer.slice(..));
                renderpass.pop_debug_group();
            }
            renderpass.insert_debug_marker("Draw!");
            renderpass.draw(0..vertices.count, 0..1);
        }

        // Submit the command in the queue to execute
        self.state.queue.submit(Some(encoder.finish()));
    }
}
