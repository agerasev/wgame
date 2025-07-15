use core::{cell::RefCell, mem};

use anyhow::{Context, Result};
use glam::Mat4;
use rgb::{ComponentMap, Rgba};

use crate::{
    State,
    object::Object,
    renderer::{RenderContext, Renderer},
    types::Color,
};

pub struct Frame<'a> {
    state: State<'a>,
    surface: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    renderer: RefCell<Renderer>,
}

impl<'a> Frame<'a> {
    pub(crate) fn new(state: State<'a>) -> Result<Self> {
        let surface = state
            .0
            .surface
            .get_current_texture()
            .context("Failed to acquire next swap chain texture")?;
        let view = surface
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut renderer = Renderer::default();
        let aspect_ratio = {
            let (width, height) = state.size();
            width as f32 / height as f32
        };
        let xform = Mat4::orthographic_rh(-aspect_ratio, aspect_ratio, -1.0, 1.0, -1.0, 1.0);
        renderer.set_xform(xform);

        Ok(Frame {
            state,
            surface,
            view,
            renderer: RefCell::new(renderer),
        })
    }

    pub fn clear(&self, color: impl Color) {
        self.renderer.borrow_mut().clear();

        let color = {
            let Rgba { r, g, b, a } = color.to_rgba().map(|c| c.to_f64());
            wgpu::Color { r, g, b, a }
        };

        let mut encoder = self
            .state
            .0
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

        self.state.0.queue.submit(Some(encoder.finish()));
    }

    pub fn add<T: Object>(&self, object: T) {
        self.renderer.borrow_mut().enqueue_object(object);
    }

    pub fn present(self) {
        mem::take(&mut *self.renderer.borrow_mut()).render(RenderContext {
            state: &self.state,
            view: &self.view,
        });
        self.surface.present()
    }
}
