use anyhow::{Context, Result};
use glam::Mat4;
use wgame_common::Frame as CommonFrame;

use crate::{object::Object, surface::State};

pub struct Frame<'a, 'b, F: CommonFrame> {
    state: &'b State<'a>,
    common: F,
    inner: Option<wgpu::SurfaceTexture>,
    view: wgpu::TextureView,
}

impl<'a, 'b, F: CommonFrame> Frame<'a, 'b, F> {
    pub(crate) fn new(state: &'b State<'a>, common: F) -> Result<Self> {
        let frame = state
            .surface
            .get_current_texture()
            .context("Failed to acquire next swap chain texture")?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        Ok(Frame {
            state,
            common,
            inner: Some(frame),
            view,
        })
    }
}

impl<F: CommonFrame> Drop for Frame<'_, '_, F> {
    fn drop(&mut self) {
        self.common.pre_present();
        self.inner
            .take()
            .expect("Inner frame is already taken")
            .present();
    }
}

impl<F: CommonFrame> Frame<'_, '_, F> {
    pub fn render<T: Object>(&mut self, object: &T) {
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
