use crate::{Graphics, Target};

/// Reusable render target without a window. Submitted pixels can be sampled or
/// copied from [`Offscreen::texture`]. Resize by constructing a new target.
pub struct Offscreen {
    state: Graphics,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
}
impl Offscreen {
    /// Panics for zero dimensions or sizes unsupported by the device.
    pub fn new(state: &Graphics, size: (u32, u32)) -> Self {
        assert!(
            size.0 > 0 && size.1 > 0,
            "Offscreen dimensions must be positive"
        );
        let texture = state.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: state.format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let encoder = state.device().create_command_encoder(&Default::default());
        Self {
            state: state.clone(),
            texture,
            view,
            encoder,
        }
    }
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
    /// Finish pending commands without submitting them, and start a fresh encoder.
    /// A host can submit this buffer before compositing the texture in one frame.
    pub fn finish(&mut self) -> wgpu::CommandBuffer {
        let encoder = std::mem::replace(
            &mut self.encoder,
            self.state
                .device()
                .create_command_encoder(&Default::default()),
        );
        encoder.finish()
    }
    /// Drop pending commands without changing already-submitted pixels.
    pub fn discard(&mut self) {
        self.encoder = self
            .state
            .device()
            .create_command_encoder(&Default::default());
    }
    /// Submit commands and start a fresh encoder. Existing pixels are preserved.
    pub fn submit(&mut self) -> wgpu::SubmissionIndex {
        let buffer = self.finish();
        self.state.queue().submit(Some(buffer))
    }
}
impl Target for Offscreen {
    fn state(&self) -> &Graphics {
        &self.state
    }
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
    fn encoder(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.encoder
    }
}
