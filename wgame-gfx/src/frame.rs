use anyhow::{Context as _, Result};

use crate::{Graphics, Surface, Target};

pub struct Frame<'a, 'b> {
    owner: &'b mut Surface<'a>,
    surface: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
    before: Vec<wgpu::CommandBuffer>,
}

impl<'a, 'b> Frame<'a, 'b> {
    pub(crate) fn new(owner: &'b mut Surface<'a>) -> Result<Self> {
        let surface = owner
            .take_texture()
            .context("Failed to acquire next swap chain texture")?;
        let view = surface
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = owner
            .state()
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut frame = Frame {
            owner,
            surface,
            view,
            encoder,
            before: Vec::new(),
        };
        frame.clear_depth();
        Ok(frame)
    }

    /// Queue prerequisite commands for the same submission as this frame.
    /// Dropping the frame without presenting also drops these commands.
    pub fn submit_before(&mut self, buffers: impl IntoIterator<Item = wgpu::CommandBuffer>) {
        self.before.extend(buffers);
    }

    pub fn present(self) {
        self.owner
            .state()
            .queue()
            .submit(self.before.into_iter().chain(Some(self.encoder.finish())));
        self.owner.state().queue().present(self.surface);
    }
}

impl Target for Frame<'_, '_> {
    fn state(&self) -> &Graphics {
        self.owner.state()
    }
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
    fn depth_view(&self) -> &wgpu::TextureView {
        self.owner.depth_view()
    }
    fn encoder(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.encoder
    }
}
