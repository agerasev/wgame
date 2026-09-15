//! Depth shared by window frames, offscreen targets and all scene renderers.
use crate::Graphics;

/// Depth format used by built-in targets and pipelines (including WebGL2).
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

/// Depth policy for a drawable. Smaller projected depth is nearer.
///
/// `ReadWrite` is the shape default. Equal depths pass, preserving scene order.
/// Use `ReadOnly` for translucent content, drawn back to front after opaque
/// content. `Overlay` ignores depth and does not write it. Alpha blending alone
/// does not prevent depth writes; fully transparent shape fragments are discarded.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DepthMode {
    #[default]
    ReadWrite,
    ReadOnly,
    Overlay,
}
impl DepthMode {
    /// Pipeline state compatible with every built-in target.
    pub fn state(self) -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(self == Self::ReadWrite),
            depth_compare: Some(if self == Self::Overlay {
                wgpu::CompareFunction::Always
            } else {
                wgpu::CompareFunction::LessEqual
            }),
            stencil: Default::default(),
            bias: Default::default(),
        }
    }
}

/// Reusable depth attachment. Recreate it when the target size changes.
/// Clear to 1 before its first use and at the beginning of each new scene/frame.
pub struct DepthBuffer {
    view: wgpu::TextureView,
}
impl DepthBuffer {
    pub fn new(state: &Graphics, size: (u32, u32)) -> Self {
        assert!(
            size.0 > 0 && size.1 > 0,
            "depth dimensions must be positive"
        );
        let texture = state.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        Self {
            view: texture.create_view(&Default::default()),
        }
    }
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}
