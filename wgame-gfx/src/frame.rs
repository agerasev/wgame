use anyhow::{Context as _, Result};
use glam::Mat4;
use rgb::{ComponentMap, Rgba};

use crate::{
    Collector, Context, Instance, Surface,
    types::{Color, color},
};

pub struct Frame<'a, 'b> {
    owner: &'b mut Surface<'a>,
    surface: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    render_passes: Collector,
    clear_color: Option<wgpu::Color>,
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
            render_passes: Collector::default(),
            clear_color: Some(wgpu::Color::BLACK),
            ctx,
        })
    }

    /// Set clear color
    pub fn clear(&mut self, color: impl Color) {
        self.clear_color = {
            let Rgba { r, g, b, a } = color.to_rgba().map(|c| c.to_f64());
            Some(wgpu::Color { r, g, b, a })
        };
    }

    pub fn push<T: Instance>(&mut self, instance: T) {
        self.render_passes.push_any(&self.ctx, instance);
    }

    pub fn render(&mut self) -> Result<usize> {
        let mut encoder = self
            .owner
            .state
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        if let Some(clear_color) = self.clear_color.take() {
            let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
        }

        let mut renderers: Vec<_> = self.render_passes.renderers().collect::<Result<_>>()?;
        renderers.sort_by(|a, b| a.order().cmp(&b.order()).then_with(|| a.cmp(b)));
        let n_passes = renderers.len();
        for renderer in renderers {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                ..Default::default()
            });
            renderer.draw(&mut pass)?;
        }
        self.render_passes = Collector::default();
        self.owner.state.queue().submit(Some(encoder.finish()));

        Ok(n_passes)
    }

    pub fn present(mut self) {
        self.render().expect("Error during rendering");
        self.surface.present();
    }
}
