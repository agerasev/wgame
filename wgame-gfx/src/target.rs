use crate::{AutoScene, Camera, Context, Graphics, Renderer, types::Color};
use crate::{Viewport, ViewportError, viewport::Region};

/// Color and depth render target. All built-in targets use [`crate::DEPTH_FORMAT`].
/// `clear` resets color and depth; successive render calls share depth. Clear
/// depth explicitly when starting an independent camera/overlay scene. Custom
/// pipelines used by `render` must declare the same depth format, even when
/// ignoring depth (see [`crate::DepthMode::Overlay`]). Borrow a [`Viewport`] to
/// draw into a smaller rectangle without allocating another render texture.
pub trait Target {
    fn state(&self) -> &Graphics;
    fn view(&self) -> &wgpu::TextureView;
    fn depth_view(&self) -> &wgpu::TextureView;
    fn encoder(&mut self) -> &mut wgpu::CommandEncoder;

    fn size(&self) -> (u32, u32) {
        let extent = self.view().texture().size();
        (extent.width, extent.height)
    }

    /// Origin of this target in its underlying attachments, in physical pixels.
    /// Custom targets overriding this must also override `size`; their rectangle
    /// must be nonempty and contained in both attachments.
    fn origin(&self) -> (u32, u32) {
        (0, 0)
    }

    /// Borrow a viewport in parent-relative physical pixels.
    /// Returns an error for empty rectangles, overflow, or bounds outside the
    /// parent. No allocation or clearing occurs until drawing is requested.
    /// See [`Viewport`] for nesting, cameras, picking, and presentation.
    fn viewport(
        &mut self,
        origin: (u32, u32),
        size: (u32, u32),
    ) -> Result<Viewport<'_, Self>, ViewportError> {
        Viewport::new(self, origin, size)
    }

    /// Replace color and reset depth to 1 inside this target's rectangle.
    /// Whole attachments use a load clear; partial regions use a clipped draw.
    fn clear(&mut self, color: impl Color) {
        crate::clear::clear(self, Some(color.to_rgba()));
    }

    /// Reset depth to 1 inside this target's rectangle while preserving color.
    fn clear_depth(&mut self) {
        crate::clear::clear(self, None);
    }

    fn render<C: Context, R: Renderer<C> + ?Sized>(&mut self, ctx: &C, renderer: &R) {
        self.render_iter(ctx, std::iter::once(renderer));
    }
    /// Encode all supplied renderers in one pass; clearing uses a separate pass.
    fn render_iter<'r, C: Context, I: Iterator<Item = &'r R>, R: Renderer<C> + ?Sized + 'r>(
        &mut self,
        ctx: &C,
        renderers: I,
    ) {
        let mut renderers = renderers.peekable();
        if renderers.peek().is_none() {
            return;
        }
        let region = Region {
            origin: self.origin(),
            size: self.size(),
        };
        let depth = self.depth_view().clone();
        let view = self.view().clone();
        let mut pass = self
            .encoder()
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
        region.apply(&mut pass);
        for renderer in renderers {
            renderer.render(ctx, &mut pass);
        }
    }

    /// Camera with Y pointing upward, visible Y from -1 to 1, and X from minus
    /// to plus the target aspect ratio.
    fn camera(&mut self) -> Camera {
        let aspect_ratio = {
            let (width, height) = self.size();
            width as f32 / height as f32
        };
        let view = glam::camera::rh::proj::directx::orthographic(
            -aspect_ratio,
            aspect_ratio,
            -1.0,
            1.0,
            -1.0,
            1.0,
        );
        Camera::new(self.state(), view)
    }
    /// Camera in local physical pixels: top-left origin, X rightward, Y downward.
    /// These pixels include the display scaling factor; they are not logical units.
    fn physical_camera(&mut self) -> Camera {
        let (width, height) = self.size();
        let view = glam::camera::lh::proj::directx::orthographic(
            0.0,
            width as f32,
            height as f32,
            0.0,
            -1.0,
            1.0,
        );
        Camera::new(self.state(), view)
    }

    fn scene(&mut self) -> AutoScene<'_, Self> {
        let camera = self.camera();
        AutoScene::new(self, camera)
    }
}
