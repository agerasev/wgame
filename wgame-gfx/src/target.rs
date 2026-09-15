use crate::{AutoScene, Camera, Context, Graphics, Renderer, types::Color};
use rgb::{ComponentMap, Rgba};

/// Color and depth render target. All built-in targets use [`crate::DEPTH_FORMAT`].
/// `clear` resets color and depth; successive render calls share depth. Clear
/// depth explicitly when starting an independent camera/overlay scene. Custom
/// pipelines used by `render` must declare the same depth format, even when
/// ignoring depth (see [`crate::DepthMode::Overlay`]).
pub trait Target {
    fn state(&self) -> &Graphics;
    fn view(&self) -> &wgpu::TextureView;
    fn depth_view(&self) -> &wgpu::TextureView;
    fn encoder(&mut self) -> &mut wgpu::CommandEncoder;

    fn size(&self) -> (u32, u32) {
        let extent = self.view().texture().size();
        (extent.width, extent.height)
    }

    fn clear(&mut self, color: impl Color) {
        let clear_color = {
            let Rgba { r, g, b, a } = color.to_rgba().map(|c| c as f64);
            wgpu::Color { r, g, b, a }
        };

        let depth = self.depth_view().clone();
        let view = &self.view().clone();
        let _ = self
            .encoder()
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
    }

    /// Reset depth while preserving color, for a new independent scene.
    fn clear_depth(&mut self) {
        let depth = self.depth_view().clone();
        let _pass = self
            .encoder()
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
    }

    fn render<C: Context, R: Renderer<C> + ?Sized>(&mut self, ctx: &C, renderer: &R) {
        let depth = self.depth_view().clone();
        let view = &self.view().clone();
        let mut pass = self
            .encoder()
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
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
        renderer.render(ctx, &mut pass);
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
    /// Camera in physical pixels: top-left origin, X rightward, Y downward.
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
