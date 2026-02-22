use crate::{AutoScene, Camera, Context, Graphics, Renderer, types::Color};
use glam::Mat4;
use rgb::{ComponentMap, Rgba};

/// Render target
pub trait Target {
    fn state(&self) -> &Graphics;
    fn view(&self) -> &wgpu::TextureView;
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
                ..Default::default()
            });
    }

    fn render<C: Context, R: Renderer<C> + ?Sized>(&mut self, ctx: &C, renderer: &R) {
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
                ..Default::default()
            });
        renderer.render(ctx, &mut pass);
    }
    fn render_iter<'r, C: Context, I: Iterator<Item = &'r R>, R: Renderer<C> + ?Sized + 'r>(
        &mut self,
        ctx: &C,
        renderers: I,
    ) {
        for renderer in renderers {
            self.render(ctx, renderer);
        }
    }

    fn camera(&mut self) -> Camera {
        let aspect_ratio = {
            let (width, height) = self.size();
            width as f32 / height as f32
        };
        let view = Mat4::orthographic_rh(-aspect_ratio, aspect_ratio, -1.0, 1.0, -1.0, 1.0);
        Camera::new(self.state(), view)
    }
    fn physical_camera(&mut self) -> Camera {
        let (width, height) = self.size();
        let view = Mat4::orthographic_lh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);
        Camera::new(self.state(), view)
    }

    fn scene(&mut self) -> AutoScene<'_, Self> {
        let camera = self.camera();
        AutoScene::new(self, camera)
    }
}
