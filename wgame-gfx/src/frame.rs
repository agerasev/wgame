use anyhow::{Context as _, Result};
use glam::Mat4;
use rgb::{ComponentMap, Rgba};

use crate::{
    Context, Instance, Renderer, Surface,
    queue::RenderQueue,
    types::{Color, color},
};

pub struct Frame<'a, 'b> {
    owner: &'b mut Surface<'a>,
    surface: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    render_passes: RenderQueue,
    clear_color: wgpu::Color,
    ctx: Context,
}

impl<'a, 'b> Frame<'a, 'b> {
    pub(crate) fn new(owner: &'b mut Surface<'a>) -> Result<Self> {
        let surface = owner
            .surface
            .get_current_texture()
            .context("Failed to acquire next swap chain texture")?;
        let view = surface
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let ctx = {
            let aspect_ratio = {
                let (width, height) = owner.size();
                width as f32 / height as f32
            };
            let view = Mat4::orthographic_rh(-aspect_ratio, aspect_ratio, -1.0, 1.0, -1.0, 1.0);
            Context {
                view,
                color: color::WHITE.to_rgba(),
            }
        };

        Ok(Frame {
            owner,
            surface,
            view,
            render_passes: RenderQueue::default(),
            clear_color: wgpu::Color::BLACK,
            ctx,
        })
    }

    /// Set clear color
    pub fn clear(&mut self, color: impl Color) {
        self.clear_color = {
            let Rgba { r, g, b, a } = color.to_rgba().map(|c| c.to_f64());
            wgpu::Color { r, g, b, a }
        };
    }

    pub fn push<T: Instance>(&mut self, instance: T) {
        self.render_passes.push(&self.ctx, instance);
    }

    fn render(&self) -> Result<()> {
        let mut encoder = self
            .owner
            .state
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        for (renderer, instances) in self.render_passes.iter() {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            renderer.draw(instances, &mut pass)?;
        }

        self.owner.state.queue().submit(Some(encoder.finish()));

        Ok(())
    }

    pub fn present(self) {
        self.render().expect("Error during rendering");
        self.surface.present();
    }
}
