use crate::{
    SampledTexture, TextureSample, TextureSettings, TexturingState,
    gpu::GpuTexture,
    sampling::{TextureRegion, TextureResource},
};
use euclid::default::Size2D;
use glam::Affine2;
use std::rc::Rc;
use wgame_gfx::{Graphics, Offscreen, Target};

/// Invalid render texture dimensions or format.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RenderTextureError {
    #[error("render texture dimensions must be positive and within the device limit")]
    InvalidSize,
    #[error("unsupported render texture format: {0:?}")]
    UnsupportedFormat(wgpu::TextureFormat),
}

/// GPU-owned color texture with depth, ordinary target drawing, and sampling.
///
/// Uses the supplied graphics format so the same shapes and materials can draw
/// into it and the window. Supported formats are RGBA/BGRA8 (linear or sRGB) and
/// RGBA16Float. A sample uses the whole texture, without an atlas or CPU pixels.
///
/// Call [`Self::submit`] before a later submission samples the result, or use
/// [`Self::finish`] with [`wgame_gfx::Frame::submit_before`] to submit both together.
/// Dropping this target discards pending commands; sampling handles retain the
/// allocation and its submitted contents. Never sample an allocation while it is
/// an attachment in the same pass; use two render textures for feedback effects.
///
/// Resize by constructing a replacement. Existing samples and baked scenes keep
/// the previous allocation. New targets start transparent black with depth 1.
///
/// Colors passed to ordinary drawing/clearing APIs use straight alpha. The GPU
/// allocation stores premultiplied RGB for correct compositing; built-in sampling
/// and readback convert back to straight alpha. Custom pipelines writing into
/// this target must maintain that representation too.
///
/// Pixel editing belongs to [`crate::Texture`], not this type:
/// ```compile_fail
/// # fn edit(texture: &wgame_gfx_texture::RenderTexture) {
/// texture.update(|pixels| {});
/// # }
/// ```
pub struct RenderTexture {
    target: Offscreen,
    sampling: Rc<GpuTexture>,
    settings: TextureSettings,
}
impl RenderTexture {
    pub fn new(
        state: &TexturingState,
        size: (u32, u32),
        settings: TextureSettings,
    ) -> Result<Self, RenderTextureError> {
        let limit = state.device().limits().max_texture_dimension_2d;
        if size.0 == 0 || size.1 == 0 || size.0 > limit || size.1 > limit {
            return Err(RenderTextureError::InvalidSize);
        }
        use wgpu::TextureFormat::*;
        let format = state.format();
        if !matches!(
            format,
            Rgba8Unorm | Rgba8UnormSrgb | Bgra8Unorm | Bgra8UnormSrgb | Rgba16Float
        ) {
            return Err(RenderTextureError::UnsupportedFormat(format));
        }
        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC;
        let features = state.adapter().get_texture_format_features(format);
        if !features.allowed_usages.contains(usage)
            || !features
                .flags
                .contains(wgpu::TextureFormatFeatureFlags::FILTERABLE)
        {
            return Err(RenderTextureError::UnsupportedFormat(format));
        }
        let mut target = Offscreen::new(state, size);
        target.clear(glam::Vec4::ZERO);
        target.submit();
        let sampling = Rc::new(GpuTexture::from_texture(
            state,
            target.texture().clone(),
            true,
        ));
        Ok(Self {
            target,
            sampling,
            settings,
        })
    }

    /// Finish pending drawing without submitting it; begin a fresh command batch.
    pub fn finish(&mut self) -> wgpu::CommandBuffer {
        self.target.finish()
    }
    /// Submit pending drawing; begin a fresh command batch.
    pub fn submit(&mut self) -> wgpu::SubmissionIndex {
        self.target.submit()
    }
    /// Discard pending drawing, preserving previously submitted contents.
    pub fn discard(&mut self) {
        self.target.discard();
    }

    /// Submit pending drawing and asynchronously download a detached image.
    /// Pixels are RGBA half floats, top row first, with normalized 8-bit channels
    /// or RGBA16Float values. RGB is converted to straight alpha and sRGB formats
    /// are decoded, matching GPU sampling and the default linear CPU texture atlas.
    /// Transparent pixels have zero RGB.
    ///
    /// Editing the returned image never updates this target. Cancellation after
    /// submission does not undo drawing. No CPU mirror or automatic readback is kept.
    pub async fn readback(
        &mut self,
    ) -> Result<wgame_image::Image<rgb::Rgba<half::f16>>, crate::ReadbackError> {
        crate::readback::readback(&mut self.target).await
    }
}
impl Target for RenderTexture {
    fn premultiplied_alpha(&self) -> bool {
        true
    }
    fn state(&self) -> &Graphics {
        self.target.state()
    }
    fn view(&self) -> &wgpu::TextureView {
        self.target.view()
    }
    fn depth_view(&self) -> &wgpu::TextureView {
        self.target.depth_view()
    }
    fn encoder(&mut self) -> &mut wgpu::CommandEncoder {
        self.target.encoder()
    }
}
impl TextureRegion for Size2D<u32> {
    fn size(&self) -> Size2D<u32> {
        *self
    }
    fn coord_xform(&self) -> Affine2 {
        Affine2::IDENTITY
    }
}
impl SampledTexture for RenderTexture {
    fn sample(&self) -> TextureSample {
        let (width, height) = self.size();
        TextureSample::new(
            TextureResource::new(self.sampling.clone(), self.settings),
            Rc::new(Size2D::new(width, height)),
        )
    }
}
