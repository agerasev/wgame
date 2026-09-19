//! A replaceable host for one interactive drawing area.
//!
//! A plain [`crate::Window`] and a UI plugin implement the same [`WindowHost`]
//! contract. The content owns its simulation; the host owns layout, input routing,
//! composition and presentation. Raw OS events remain on [`crate::Window::input`].
//!
//! ```no_run
//! use wgame::{WindowHost, ContentFrame, prelude::*, gfx::types::color};
//! async fn run(mut host: impl WindowHost) -> wgame::Result<()> {
//!     while let Some(mut frame) = host.next_frame().await? {
//!         let _input = frame.input();
//!         frame.clear(color::BLACK);
//!         frame.present();
//!     }
//!     Ok(())
//! }
//! ```

use crate::{Result, gfx};
use wgame_input::CanvasInput;

/// A host yields at most one borrowed content frame at a time. UI plugins may
/// reserve part of the OS window; content dimensions and input are local to it.
/// `None` means the window closed, not that the content is temporarily hidden.
pub trait WindowHost {
    type Frame<'a>: ContentFrame
    where
        Self: 'a;
    fn graphics(&self) -> &gfx::Graphics;
    /// Wait for input, a window/UI repaint request, or an application deadline.
    /// `None` waits indefinitely; `Some(Duration::ZERO)` returns immediately.
    /// Call before `next_frame` to avoid continuously drawing static content.
    /// Draw an initial frame before waiting indefinitely.
    /// It does not consume the next frame's input. Cancellation is safe.
    /// Hosts without event-driven support may return immediately.
    fn wait_for_update(
        &mut self,
        _timeout: Option<std::time::Duration>,
    ) -> impl Future<Output = ()> {
        async {}
    }
    fn next_frame(&mut self) -> impl Future<Output = Result<Option<Self::Frame<'_>>>>;
}

/// Content rendering and input for one application update. Normal drop presents;
/// explicit discard and panic unwinding do not submit content or window commands.
/// Finish borrowed scenes before presenting. Simulation work belongs outside the
/// host's UI layout callbacks, which may run more than once.
pub trait ContentFrame: gfx::Target {
    /// Whether content is visible. Hidden content still yields frames so the
    /// application can process UI actions; its backing target remains nonzero.
    fn visible(&self) -> bool {
        true
    }
    fn input(&self) -> &CanvasInput;
    /// Effective physical pixels per local logical pixel (including host UI zoom).
    fn scale_factor(&self) -> f64;
    fn resized(&self) -> Option<(u32, u32)>;
    fn logical_size(&self) -> (f64, f64) {
        let (width, height) = self.size();
        (
            width as f64 / self.scale_factor(),
            height as f64 / self.scale_factor(),
        )
    }
    fn logical_camera(&mut self) -> gfx::Camera {
        let (width, height) = self.logical_size();
        gfx::Camera::new(
            self.state(),
            glam::camera::lh::proj::directx::orthographic(
                0.0,
                width as f32,
                height as f32,
                0.0,
                -1.0,
                1.0,
            ),
        )
    }
    fn present(self);
    fn discard(self);
}
