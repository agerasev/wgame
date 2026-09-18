//! Optional egui host for one wgame drawing area.
//!
//! Wrap a window, build controls in the layout callback, and hand the result to
//! code generic over [`WindowHost`]. The callback returns the response from
//! [`Canvas::show`]; its widget ID stays stable across frames. Input and camera
//! coordinates are local logical pixels, including egui zoom. Clicking the canvas
//! gives it keyboard focus. UI controls own their events; a captured canvas drag
//! retains its release outside the canvas. Geometry/focus loss cancels gestures.
//! Lifting a touch preserves its completed release and clears pointer hover;
//! a cancelled touch aborts the gesture instead.
//!
//! ```no_run
//! use wgame::{ContentFrame, WindowHost, gfx::{Target, types::color}};
//! use wgame_egui::{egui, EguiWindow};
//! async fn run(window: wgame::Window<'_>) -> wgame::Result<()> {
//!     let mut host = EguiWindow::new(window, |ui, canvas| {
//!         egui::Panel::top("toolbar").show(ui, |ui| { ui.label("My playground"); });
//!         egui::CentralPanel::default().show(ui, |ui| canvas.show(ui)).inner
//!     });
//!     while let Some(mut frame) = host.next_frame().await? {
//!         frame.clear(color::BLACK);
//!         // Update once here, then render with frame.logical_camera().
//!     }
//!     Ok(())
//! }
//! ```
//!
//! Create renderers from the wrapper's [`WindowHost::graphics`]: its RGBA canvas
//! target can have a different format from the OS surface. Composition uses one
//! offscreen texture and an additional pass. Layout callbacks may run repeatedly;
//! keep simulation updates outside them. Normal frame drop presents once;
//! discard/panic drops all encoded commands (texture uploads may already occur).
//! A hidden/empty canvas yields a minimal target with `visible() == false`, so
//! application updates and UI actions can still run.
//!
//! This integration owns one OS window. Native secondary viewports and viewport
//! screenshot/clipboard-image requests are not supported. Use embedded egui
//! windows instead. The `desktop` feature enables native clipboard and links;
//! `web` uses WebGL2 and a winit event adapter for pointer, touch, keyboard and
//! wheel input. Browser text entry requires a hardware keyboard: mobile virtual
//! keyboards, IME, system clipboard and file drops are not integrated. The web
//! adapter provides an in-app text clipboard.

#![forbid(unsafe_code)]

mod input;
#[cfg(not(target_arch = "wasm32"))]
use egui_winit as platform;
#[cfg(any(target_arch = "wasm32", test))]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod web;
pub use egui;
use input::CanvasState;
#[cfg(target_arch = "wasm32")]
use web as platform;
use wgame::{
    ContentFrame, WindowHost,
    canvas::CanvasInput,
    gfx::{self, Target},
};

/// The content texture, allocated to the available UI rectangle.
pub struct Canvas {
    texture: egui::TextureId,
}
impl Canvas {
    /// Place exactly once per layout pass. The canvas uses a stable widget ID.
    pub fn show(&self, ui: &mut egui::Ui) -> egui::Response {
        let (_, rect) = ui.allocate_space(ui.available_size().max(egui::Vec2::ZERO));
        let response = ui.interact(
            rect,
            egui::Id::new("wgame-canvas"),
            egui::Sense::click_and_drag(),
        );
        ui.painter().image(
            self.texture,
            rect,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        if response.clicked() || response.drag_started() {
            response.request_focus();
        }
        response
    }
}

struct Painter {
    target: gfx::Offscreen,
    renderer: egui_wgpu::Renderer,
    canvas: Canvas,
}
impl Painter {
    fn new(graphics: &gfx::Graphics) -> Self {
        let canvas_graphics = gfx::Graphics::new(
            graphics.adapter().clone(),
            graphics.device().clone(),
            graphics.queue().clone(),
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let target = gfx::Offscreen::new(&canvas_graphics, (1, 1));
        let mut renderer =
            egui_wgpu::Renderer::new(graphics.device(), graphics.format(), Default::default());
        let texture = renderer.register_native_texture(
            graphics.device(),
            target.view(),
            wgpu::FilterMode::Linear,
        );
        Self {
            target,
            renderer,
            canvas: Canvas { texture },
        }
    }
    fn resize(&mut self, size: (u32, u32)) {
        if self.target.size() != size {
            self.target = gfx::Offscreen::new(self.target.state(), size);
            self.renderer.update_egui_texture_from_wgpu_texture(
                self.target.state().device(),
                self.target.view(),
                wgpu::FilterMode::Linear,
                self.canvas.texture,
            );
        }
    }
    fn paint(
        &mut self,
        root: &mut impl Target,
        jobs: &[egui::ClippedPrimitive],
        screen: &egui_wgpu::ScreenDescriptor,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut before = vec![self.target.finish()];
        let graphics = root.state().clone();
        let extra = self.renderer.update_buffers(
            graphics.device(),
            graphics.queue(),
            root.encoder(),
            jobs,
            screen,
        );
        before.extend(extra);
        let view = root.view().clone();
        {
            let mut pass = root
                .encoder()
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("egui composition"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                })
                .forget_lifetime();
            self.renderer.render(&mut pass, jobs, screen);
        }
        before
    }
}

/// A window decorated with an egui layout containing one [`Canvas`].
pub struct EguiWindow<'w, L> {
    window: wgame::Window<'w>,
    events: wgame::app::Input,
    context: egui::Context,
    platform: platform::State,
    layout: L,
    painter: Painter,
    input: CanvasState,
    size: Option<(u32, u32)>,
    closed: bool,
}
impl<'w, L: FnMut(&mut egui::Ui, &Canvas) -> egui::Response> EguiWindow<'w, L> {
    pub fn new(window: wgame::Window<'w>, layout: L) -> Self {
        let context = egui::Context::default();
        context.set_embed_viewports(true);
        let mut platform = platform::State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            window.raw(),
            Some(window.scale_factor() as f32),
            window.raw().theme(),
            Some(window.graphics().device().limits().max_texture_dimension_2d as usize),
        );
        platform.egui_input_mut().focused = window.raw().has_focus();
        Self {
            events: window.input(),
            painter: Painter::new(window.graphics()),
            window,
            context,
            platform,
            layout,
            input: CanvasState::default(),
            size: None,
            closed: false,
        }
    }
    pub fn context(&self) -> &egui::Context {
        &self.context
    }
}
impl<'w, L: FnMut(&mut egui::Ui, &Canvas) -> egui::Response> WindowHost for EguiWindow<'w, L> {
    type Frame<'a>
        = Frame<'w, 'a>
    where
        Self: 'a;
    fn graphics(&self) -> &gfx::Graphics {
        self.painter.target.state()
    }
    async fn next_frame(&mut self) -> wgame::Result<Option<Self::Frame<'_>>> {
        {
            if self.closed {
                return Ok(None);
            }
            let raw_window = self.window.raw();
            let Some(root) = self.window.next_frame().await? else {
                return Ok(None);
            };
            while let Some(event) = self.events.try_next() {
                #[cfg(not(target_arch = "wasm32"))]
                let _ = self.platform.on_window_event(raw_window, &event);
                #[cfg(target_arch = "wasm32")]
                self.platform.on_window_event(raw_window, &event);
            }
            platform::update_viewport_info(
                self.platform
                    .egui_input_mut()
                    .viewports
                    .entry(egui::ViewportId::ROOT)
                    .or_default(),
                &self.context,
                raw_window,
                false,
            );
            let raw = self.platform.take_egui_input(raw_window);
            let events = raw.events.clone();
            let focused = raw.focused;
            let mut response = None;
            let mut output = self.context.run_ui(raw, |ui| {
                response = Some((self.layout)(ui, &self.painter.canvas));
            });
            // Drain first: TexturesDelta requires explicit handling even on discard.
            let updates = std::mem::take(&mut output.textures_delta.set);
            let free = std::mem::take(&mut output.textures_delta.free)
                .into_iter()
                .collect();
            for (id, deltas) in updates {
                for delta in deltas {
                    self.painter.renderer.update_texture(
                        root.state().device(),
                        root.state().queue(),
                        id,
                        &delta,
                    );
                }
            }
            self.platform
                .handle_platform_output(raw_window, output.platform_output);
            if let Some(viewport) = output.viewport_output.remove(&egui::ViewportId::ROOT) {
                self.closed |= viewport
                    .commands
                    .iter()
                    .any(|cmd| matches!(cmd, egui::ViewportCommand::Close));
                platform::process_viewport_commands(
                    &self.context,
                    self.platform
                        .egui_input_mut()
                        .viewports
                        .entry(egui::ViewportId::ROOT)
                        .or_default(),
                    viewport.commands,
                    raw_window,
                    &mut Vec::new(),
                );
            }
            let response = response.expect("egui executes a layout pass");
            let modifiers = self.context.input(|i| i.modifiers);
            let scale = output.pixels_per_point;
            let mut input = self
                .input
                .collect(&response, &events, focused, modifiers, scale);
            if input.focused
                && !input
                    .events
                    .iter()
                    .any(|event| matches!(event, wgame::canvas::Event::Cancelled))
            {
                input.relative_motion = root.input().relative_motion;
            }
            let visible = response.rect.is_positive() && response.interact_rect.is_positive();
            let limit = root.state().device().limits().max_texture_dimension_2d;
            let size = (
                (response.rect.width() * scale)
                    .round()
                    .clamp(1.0, limit as f32) as u32,
                (response.rect.height() * scale)
                    .round()
                    .clamp(1.0, limit as f32) as u32,
            );
            let resized = (self.size != Some(size)).then_some(size);
            self.size = Some(size);
            self.painter.resize(size);
            let jobs = self.context.tessellate(output.shapes, scale);
            let (width, height) = root.size();
            let screen = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [width, height],
                pixels_per_point: scale,
            };
            let frame = Frame {
                root: Some(root),
                painter: &mut self.painter,
                jobs,
                screen,
                free,
                input,
                resized,
                logical_size: (
                    response.rect.width().max(1.0 / scale) as f64,
                    response.rect.height().max(1.0 / scale) as f64,
                ),
                visible,
            };
            Ok(Some(frame))
        }
    }
}

/// Borrowed canvas target; presentation composites it with the UI.
pub struct Frame<'w, 'a> {
    root: Option<wgame::Frame<'w, 'a>>,
    painter: &'a mut Painter,
    jobs: Vec<egui::ClippedPrimitive>,
    screen: egui_wgpu::ScreenDescriptor,
    free: Vec<egui::TextureId>,
    input: CanvasInput,
    resized: Option<(u32, u32)>,
    logical_size: (f64, f64),
    visible: bool,
}
impl Target for Frame<'_, '_> {
    fn state(&self) -> &gfx::Graphics {
        self.painter.target.state()
    }
    fn view(&self) -> &wgpu::TextureView {
        self.painter.target.view()
    }
    fn depth_view(&self) -> &wgpu::TextureView {
        self.painter.target.depth_view()
    }
    fn encoder(&mut self) -> &mut wgpu::CommandEncoder {
        self.painter.target.encoder()
    }
}
impl ContentFrame for Frame<'_, '_> {
    fn visible(&self) -> bool {
        self.visible
    }
    fn input(&self) -> &CanvasInput {
        &self.input
    }
    fn scale_factor(&self) -> f64 {
        self.screen.pixels_per_point as f64
    }
    fn logical_size(&self) -> (f64, f64) {
        self.logical_size
    }
    fn resized(&self) -> Option<(u32, u32)> {
        self.resized
    }
    fn present(mut self) {
        self.finish(true);
    }
    fn discard(mut self) {
        self.finish(false);
    }
}
impl Frame<'_, '_> {
    fn finish(&mut self, present: bool) {
        let Some(mut root) = self.root.take() else {
            return;
        };
        if present {
            let before = self.painter.paint(&mut root, &self.jobs, &self.screen);
            root.submit_before(before);
            root.present();
        } else {
            self.painter.target.discard();
            root.discard();
        }
        for id in self.free.drain(..) {
            self.painter.renderer.free_texture(&id);
        }
    }
}
impl Drop for Frame<'_, '_> {
    fn drop(&mut self) {
        self.finish(!std::thread::panicking());
    }
}

#[cfg(test)]
mod rendering_tests;
