use anyhow::{Context as _, Result};

use crate::{Graphics, Surface, Target};

pub struct Frame<'a, 'b> {
    owner: &'b mut Surface<'a>,
    surface: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
}

impl<'a, 'b> Frame<'a, 'b> {
    pub(crate) fn new(owner: &'b mut Surface<'a>) -> Result<Self> {
        let surface = owner
            .inner()
            .get_current_texture()
            .context("Failed to acquire next swap chain texture")?;
        let view = surface
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = owner
            .state()
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        Ok(Frame {
            owner,
            surface,
            view,
            encoder,
        })
    }

    pub fn present(self) {
        self.owner
            .state()
            .queue()
            .submit(Some(self.encoder.finish()));
        self.surface.present();
    }
}

impl Target for Frame<'_, '_> {
    fn state(&self) -> &Graphics {
        self.owner.state()
    }
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
    fn encoder(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.encoder
    }
}
