use anyhow::{Context as _, Result};
use glam::Mat4;
use rgb::{ComponentMap, Rgba};

use crate::{Camera, Collector, CollectorWithCamera, Surface, types::Color};

pub struct Frame<'a, 'b> {
    owner: &'b mut Surface<'a>,
    surface: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    collector: Collector,
    clear_color: Option<wgpu::Color>,
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

        Ok(Frame {
            owner,
            surface,
            view,
            collector: Collector::default(),
            clear_color: Some(wgpu::Color::BLACK),
        })
    }

    /// Set clear color
    pub fn clear(&mut self, color: impl Color) {
        self.clear_color = {
            let Rgba { r, g, b, a } = color.to_rgba().map(|c| c.to_f64());
            Some(wgpu::Color { r, g, b, a })
        };
    }

    pub fn collector(&mut self) -> &mut Collector {
        &mut self.collector
    }
    pub fn with_physical_camera(&mut self) -> CollectorWithCamera<'_> {
        let (width, height) = self.owner.size();
        CollectorWithCamera {
            collector: &mut self.collector,
            camera: Camera {
                view: Mat4::orthographic_lh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0),
                y_flip: true,
                ..Default::default()
            },
        }
    }
    pub fn with_unit_camera(&mut self) -> CollectorWithCamera<'_> {
        let aspect_ratio = {
            let (width, height) = self.owner.size();
            width as f32 / height as f32
        };
        CollectorWithCamera {
            collector: &mut self.collector,
            camera: Camera {
                view: Mat4::orthographic_rh(-aspect_ratio, aspect_ratio, -1.0, 1.0, -1.0, 1.0),
                ..Default::default()
            },
        }
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

        let mut n_passes = 0;
        for (resource, storage) in self.collector.items() {
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
            resource.render_dyn(storage, &mut pass);
            n_passes += 1;
        }

        self.collector = Collector::default();
        self.owner.state.queue().submit(Some(encoder.finish()));

        Ok(n_passes)
    }

    pub fn present(mut self) {
        self.render().expect("Error during rendering");
        self.surface.present();
    }
}
