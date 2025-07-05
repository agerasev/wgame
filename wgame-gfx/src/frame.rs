use anyhow::{Context, Result};
use glam::Mat4;
use rgb::{ComponentMap, Rgba};

use crate::{SharedState, object::Object, types::Color};

pub struct Frame<'a> {
    state: SharedState<'a>,
    surface: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
}

impl<'a> Frame<'a> {
    pub(crate) fn new(state: SharedState<'a>) -> Result<Self> {
        let surface = state
            .surface
            .get_current_texture()
            .context("Failed to acquire next swap chain texture")?;
        let view = surface.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(state.format.add_srgb_suffix()),
            ..Default::default()
        });
        Ok(Frame {
            state,
            surface,
            view,
        })
    }

    pub fn present(self) {
        self.surface.present()
    }

    pub fn clear(&self, color: impl Color) {
        let color = {
            let Rgba { r, g, b, a } = color.to_rgba().map(|c| c as f64);
            wgpu::Color { r, g, b, a }
        };

        let mut encoder = self
            .state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.state.queue.submit(Some(encoder.finish()));
    }

    pub fn render<T: Object>(&self, object: &T) {
        let vertices = object.vertices();

        let aspect_ratio = {
            let (width, height) = self.state.size();
            width as f32 / height as f32
        };
        let xform = Mat4::orthographic_rh(-aspect_ratio, aspect_ratio, -1.0, 1.0, -1.0, 1.0);
        let uniforms = object.create_uniforms(xform);

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
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            {
                renderpass.push_debug_group("prepare");
                renderpass.set_pipeline(&object.pipeline());
                renderpass.set_bind_group(0, &uniforms.vertex, &[]);
                renderpass.set_bind_group(1, &uniforms.fragment, &[]);
                renderpass.set_vertex_buffer(0, vertices.vertex_buffer.slice(..));
                if let Some(index_buffer) = &vertices.index_buffer {
                    renderpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                }
                renderpass.pop_debug_group();
            }
            renderpass.insert_debug_marker("draw");
            if vertices.index_buffer.is_some() {
                renderpass.draw_indexed(0..vertices.count, 0, 0..1);
            } else {
                renderpass.draw(0..vertices.count, 0..1);
            }
        }

        self.state.queue.submit(Some(encoder.finish()));
    }
}
