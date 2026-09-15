use std::ops::{Deref, DerefMut};

use anyhow::Result;

use crate::{
    app::{self, Input, Runtime},
    config::WindowConfig,
    gfx,
};

pub use app::{WindowError, WindowedTask};

pub fn create_windowed_task<T, F>(
    rt: &Runtime,
    config: WindowConfig,
    window_fn: F,
) -> WindowedTask<Result<T, gfx::Error>>
where
    T: 'static,
    F: AsyncFnOnce(Window<'_>) -> T + 'static,
{
    app::create_windowed_task(rt, config.app, async move |app_window| {
        let window = Window::new(app_window, config.gfx).await?;
        Ok(window_fn(window).await)
    })
}

pub async fn within_window<T, F>(config: WindowConfig, window_fn: F) -> Result<T>
where
    T: 'static,
    F: AsyncFnOnce(Window<'_>) -> Result<T> + 'static,
{
    create_windowed_task(&Runtime::current(), config, window_fn).await??
}

/// A window with an async drawing loop and its graphics surface.
///
/// Use `#[wgame::window(size = (800, 600))]` on an async function taking a
/// `Window<'_>`. Closing ends that function normally when it returns after
/// [`Self::next_frame`] yields `None`. Suspension cancels the function; on resume,
/// the single-window wrapper invokes it again and recreates window-local resources.
/// Keep application data outside that function if it must survive suspension.
///
/// Use `#[wgame::app]` on an async zero-argument function for multiple windows.
/// [`create_windowed_task`] yields one window's result; on [`WindowError::Suspended`],
/// its caller decides whether to recreate it.
pub struct Window<'a> {
    gfx: gfx::Surface<'a>,
    app: app::Window<'a>,
    pending_resize: Option<(u32, u32)>,
    host_input: crate::host_input::PlainInput,
    last_scale: f64,
}

impl<'a> Window<'a> {
    async fn new(app: app::Window<'a>, gfx_cfg: gfx::Config) -> Result<Self> {
        let display = Runtime::current()
            .run_within_event_loop(
                |event_loop| event_loop.owned_display_handle(),
                Default::default(),
            )
            .await;
        let mut gfx = gfx::Surface::with_display_handle(gfx_cfg, app.raw(), display).await?;
        gfx.resize(app.size());
        let pending_resize = Some(app.size());
        let host_input = crate::host_input::PlainInput::new(app.input(), app.raw().has_focus());
        let last_scale = app.scale_factor();
        Ok(Self {
            host_input,
            last_scale,
            app,
            gfx,
            pending_resize,
        })
    }

    /// Native window access for integrations handling cursor, IME and clipboard.
    /// The reference lives for the window task, so it can be used while a frame
    /// borrows the graphics surface. Resize/redraw events still belong to wgame.
    pub fn raw(&self) -> &'a app::RawWindow {
        self.app.raw()
    }

    /// OS scale factor mapping logical pixels to physical pixels.
    ///
    /// For drawing, prefer the snapshot on [`Frame::scale_factor`].
    pub fn scale_factor(&self) -> f64 {
        self.app.scale_factor()
    }

    /// Create an independent event stream; see [`Input`] for buffering and termination.
    pub fn input(&self) -> Input {
        self.app.input()
    }

    /// Wait for a drawable frame, or return `None` when the window closes.
    ///
    /// Timeouts and occlusion request another redraw; outdated/suboptimal surfaces
    /// are reconfigured. Surface loss and validation errors propagate. Zero-size
    /// surfaces are not configured, and resize notifications survive skipped frames.
    pub async fn next_frame(&mut self) -> Result<Option<Frame<'a, '_>>> {
        loop {
            let Some(redraw) = self.app.request_redraw().await else {
                return Ok(None);
            };
            if let Some(size) = redraw.resized() {
                self.gfx.resize(size);
                self.pending_resize = Some(size);
            }
            if !self.gfx.prepare_frame()? {
                continue;
            }
            let changed = self.pending_resize.is_some() || self.last_scale != redraw.scale_factor();
            let mut input = self.host_input.collect(redraw.scale_factor(), changed);
            if input.focused
                && !input
                    .events
                    .iter()
                    .any(|event| matches!(event, crate::canvas::Event::Cancelled))
            {
                let (x, y) = redraw.mouse_motion();
                input.relative_motion = glam::Vec2::new(x as f32, y as f32);
            }
            self.last_scale = redraw.scale_factor();
            return Ok(Some(Frame {
                input,
                app: redraw,
                gfx: Some(self.gfx.frame()?),
                resized: self.pending_resize.take(),
            }));
        }
    }

    pub fn graphics(&self) -> &gfx::Graphics {
        self.gfx.state()
    }
}

/// A drawing frame borrowing its window.
///
/// [`Self::present`] submits and presents once; normal drop does the same.
/// [`Self::discard`] submits nothing. Panic unwinding does not submit the frame.
/// Finish any [`gfx::AutoScene`] borrowing the frame before presenting it.
pub struct Frame<'a, 'b> {
    gfx: Option<gfx::Frame<'a, 'b>>,
    app: app::window::Redraw<'b>,
    resized: Option<(u32, u32)>,
    input: crate::canvas::CanvasInput,
}

impl<'a, 'b> Deref for Frame<'a, 'b> {
    type Target = gfx::Frame<'a, 'b>;
    fn deref(&self) -> &Self::Target {
        self.gfx.as_ref().unwrap()
    }
}

impl DerefMut for Frame<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.gfx.as_mut().unwrap()
    }
}

impl Drop for Frame<'_, '_> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            self.present_inner();
        }
    }
}

impl Frame<'_, '_> {
    /// Input local to this frame's drawing area. Raw events remain on the window.
    pub fn input(&self) -> &crate::canvas::CanvasInput {
        &self.input
    }
    fn present_inner(&mut self) {
        if let Some(frame) = self.gfx.take() {
            self.app.pre_present();
            frame.present();
        }
    }
    /// Submit and present now. Dropping a frame also presents unless unwinding.
    pub fn present(mut self) {
        self.present_inner();
    }
    /// Drop encoded commands without submitting or presenting them.
    pub fn discard(mut self) {
        self.gfx.take();
    }

    /// OS scale factor captured with this frame's physical dimensions.
    ///
    /// Compare consecutive frames to refresh resources when display scaling
    /// changes, even if [`Self::resized`] is `None`. Skipped surface acquisitions
    /// do not hide the latest factor.
    pub fn scale_factor(&self) -> f64 {
        self.app.scale_factor()
    }

    /// Inner dimensions in logical pixels (physical size / scale factor).
    pub fn logical_size(&self) -> (f64, f64) {
        self.app.logical_size()
    }

    /// Camera in logical pixels: top-left origin, X rightward, Y downward.
    ///
    /// Use this explicitly for display-scaled UI. Render targets and
    /// [`gfx::Target::physical_camera`] remain in physical pixels; scene cameras
    /// are otherwise unaffected. For picking, pass physical cursor positions and
    /// [`Self::size`] to [`gfx::Camera::screen_to_world`].
    ///
    /// Rasterize text at `logical_font_size * frame.scale_factor() as f32`,
    /// then draw it scaled by `logical_font_size` through this camera. Rebuild
    /// cached text when the raster size changes to keep glyphs sharp.
    ///
    /// ```no_run
    /// # async fn draw(mut window: wgame::Window<'_>) -> wgame::Result<()> {
    /// use wgame::{prelude::*, glam::Vec2, gfx::types::color};
    /// let library = wgame::Library::new(window.graphics());
    /// while let Some(mut frame) = window.next_frame().await? {
    ///     let camera = frame.logical_camera();
    ///     let mut scene = frame.scene();
    ///     scene.camera = camera;
    ///     scene.add(&library.shapes()
    ///         .rectangle((Vec2::splat(12.0), Vec2::new(112.0, 52.0)))
    ///         .fill_color(color::WHITE));
    /// }
    /// # Ok(()) }
    /// ```
    pub fn logical_camera(&mut self) -> gfx::Camera {
        let scale_factor = self.scale_factor();
        logical_camera(&mut **self, scale_factor)
    }

    /// Inner dimensions in physical pixels.
    pub fn size(&self) -> (u32, u32) {
        self.app.size()
    }

    pub fn resized(&self) -> Option<(u32, u32)> {
        self.resized
    }
}

fn logical_camera(target: &mut impl gfx::Target, scale_factor: f64) -> gfx::Camera {
    use gfx::prelude::Transformable;
    target
        .physical_camera()
        .transform(glam::Affine2::from_scale(glam::Vec2::splat(
            scale_factor as f32,
        )))
}

#[cfg(test)]
#[path = "window/tests.rs"]
mod tests;

impl gfx::Target for Frame<'_, '_> {
    fn state(&self) -> &gfx::Graphics {
        gfx::Target::state(&**self)
    }
    fn view(&self) -> &wgpu::TextureView {
        gfx::Target::view(&**self)
    }
    fn depth_view(&self) -> &wgpu::TextureView {
        gfx::Target::depth_view(&**self)
    }
    fn encoder(&mut self) -> &mut wgpu::CommandEncoder {
        gfx::Target::encoder(&mut **self)
    }
}
impl crate::ContentFrame for Frame<'_, '_> {
    fn input(&self) -> &crate::canvas::CanvasInput {
        self.input()
    }
    fn scale_factor(&self) -> f64 {
        self.scale_factor()
    }
    fn resized(&self) -> Option<(u32, u32)> {
        self.resized()
    }
    fn present(self) {
        self.present();
    }
    fn discard(self) {
        self.discard();
    }
}
impl<'w> crate::WindowHost for Window<'w> {
    type Frame<'a>
        = Frame<'w, 'a>
    where
        Self: 'a;
    fn graphics(&self) -> &gfx::Graphics {
        self.graphics()
    }
    async fn next_frame(&mut self) -> Result<Option<Self::Frame<'_>>> {
        self.next_frame().await
    }
}
