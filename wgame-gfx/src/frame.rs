use anyhow::{Context as _, Result};
use glam::Mat4;
use rgb::{ComponentMap, Rgba};

use crate::{Camera, Context, Renderer, Surface, types::Color};

pub struct Frame<'a, 'b> {
    owner: &'b mut Surface<'a>,
    surface: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
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
        let encoder = owner
            .state
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        Ok(Frame {
            owner,
            surface,
            view,
            encoder,
        })
    }

    pub fn physical_camera(&mut self) -> Camera {
        let (width, height) = self.owner.size();
        let view = Mat4::orthographic_lh(0.0, width as f32, 0.0, height as f32, -1.0, 1.0);
        Camera::new(&self.owner.state).transform(view)
    }
    pub fn unit_camera(&mut self) -> Camera {
        let aspect_ratio = {
            let (width, height) = self.owner.size();
            width as f32 / height as f32
        };
        let view = Mat4::orthographic_rh(-aspect_ratio, aspect_ratio, -1.0, 1.0, -1.0, 1.0);
        Camera::new(&self.owner.state).transform(view)
    }

    pub fn clear(&mut self, color: impl Color) {
        let clear_color = {
            let Rgba { r, g, b, a } = color.to_rgba().map(|c| c.to_f64());
            wgpu::Color { r, g, b, a }
        };

        let _ = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

    pub fn draw_single<C: Context, R: Renderer<C> + ?Sized>(&mut self, ctx: &C, renderer: &R) {
        let mut pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        renderer.render(ctx, &mut pass);
    }

    pub fn draw_multiple<
        'r,
        C: Context,
        I: Iterator<Item = &'r R>,
        R: Renderer<C> + ?Sized + 'r,
    >(
        &mut self,
        ctx: &C,
        renderers: I,
    ) {
        for renderer in renderers {
            self.draw_single(ctx, renderer);
        }
    }

    pub fn present(self) {
        self.owner.state.queue().submit(Some(self.encoder.finish()));
        self.surface.present();
    }
}
