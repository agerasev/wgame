//! Browser event bridge. egui-winit's native file handles do not compile on wasm;
//! keep browser behavior explicit instead of depending on native platform APIs.
use egui::{Event, Modifiers, PointerButton, ViewportInfo};
use winit::{
    event::{MouseButton, MouseScrollDelta, TouchPhase, WindowEvent},
    keyboard::Key,
    window::Window,
};

pub struct State {
    ctx: egui::Context,
    raw: egui::RawInput,
    modifiers: Modifiers,
    pointer: Option<egui::Pos2>,
    touch: Option<u64>,
    start: wgame::app::time::Instant,
    clipboard: String,
}
impl State {
    pub fn new(
        ctx: egui::Context,
        _id: egui::ViewportId,
        window: &Window,
        _scale: Option<f32>,
        _theme: Option<winit::window::Theme>,
        max_texture_side: Option<usize>,
    ) -> Self {
        Self {
            ctx,
            raw: egui::RawInput {
                focused: window.has_focus(),
                max_texture_side,
                ..Default::default()
            },
            modifiers: Default::default(),
            pointer: None,
            touch: None,
            start: wgame::app::time::Instant::now(),
            clipboard: String::new(),
        }
    }
    pub fn egui_input_mut(&mut self) -> &mut egui::RawInput {
        &mut self.raw
    }
    pub fn on_window_event(&mut self, window: &Window, event: &WindowEvent) {
        let scale = window.scale_factor() as f32 * self.ctx.zoom_factor();
        self.handle(event, scale);
    }
    fn handle(&mut self, event: &WindowEvent, scale: f32) {
        let pos = |p: winit::dpi::PhysicalPosition<f64>| {
            egui::pos2(p.x as f32 / scale, p.y as f32 / scale)
        };
        match event {
            WindowEvent::CursorMoved { position, .. } if self.touch.is_none() => {
                let p = pos(*position);
                self.pointer = Some(p);
                self.raw.events.push(Event::PointerMoved(p));
            }
            WindowEvent::CursorLeft { .. } if self.touch.is_none() => {
                self.pointer = None;
                self.raw.events.push(Event::PointerGone);
            }
            WindowEvent::MouseInput { state, button, .. } if self.touch.is_none() => {
                let button = match button {
                    MouseButton::Left => Some(PointerButton::Primary),
                    MouseButton::Right => Some(PointerButton::Secondary),
                    MouseButton::Middle => Some(PointerButton::Middle),
                    MouseButton::Back => Some(PointerButton::Extra1),
                    MouseButton::Forward => Some(PointerButton::Extra2),
                    _ => None,
                };
                if let (Some(pos), Some(button)) = (self.pointer, button) {
                    self.raw.events.push(Event::PointerButton {
                        pos,
                        button,
                        pressed: state.is_pressed(),
                        modifiers: self.modifiers,
                    });
                }
            }
            WindowEvent::Touch(touch) => {
                let p = pos(touch.location);
                match touch.phase {
                    TouchPhase::Started if self.touch.is_none() => {
                        self.touch = Some(touch.id);
                        self.pointer = Some(p);
                        self.raw.events.push(Event::PointerMoved(p));
                        self.raw.events.push(Event::PointerButton {
                            pos: p,
                            button: PointerButton::Primary,
                            pressed: true,
                            modifiers: self.modifiers,
                        });
                    }
                    TouchPhase::Moved if self.touch == Some(touch.id) => {
                        self.pointer = Some(p);
                        self.raw.events.push(Event::PointerMoved(p));
                    }
                    TouchPhase::Ended if self.touch == Some(touch.id) => {
                        self.raw.events.push(Event::PointerButton {
                            pos: p,
                            button: PointerButton::Primary,
                            pressed: false,
                            modifiers: self.modifiers,
                        });
                        self.raw.events.push(Event::PointerGone);
                        self.touch = None;
                        self.pointer = None;
                    }
                    TouchPhase::Cancelled if self.touch == Some(touch.id) => {
                        self.raw.events.push(Event::PointerGone);
                        self.touch = None;
                        self.pointer = None;
                    }
                    _ => {}
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (unit, delta) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        (egui::MouseWheelUnit::Line, egui::vec2(*x, *y))
                    }
                    MouseScrollDelta::PixelDelta(p) => (
                        egui::MouseWheelUnit::Point,
                        egui::vec2(p.x as f32, p.y as f32) / scale,
                    ),
                };
                self.raw.events.push(Event::MouseWheel {
                    unit,
                    delta,
                    phase: egui::TouchPhase::Move,
                    modifiers: self.modifiers,
                });
            }
            WindowEvent::ModifiersChanged(m) => {
                let m = m.state();
                self.modifiers = Modifiers {
                    alt: m.alt_key(),
                    ctrl: m.control_key(),
                    shift: m.shift_key(),
                    mac_cmd: m.super_key(),
                    command: m.control_key() || m.super_key(),
                };
                self.raw
                    .events
                    .push(Event::ModifiersChanged(self.modifiers));
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let key = match &event.logical_key {
                    Key::Character(s) => egui::Key::from_name(s),
                    Key::Named(k) => egui::Key::from_name(&format!("{k:?}")),
                    _ => None,
                };
                if let Some(key) = key {
                    if event.state.is_pressed() && self.modifiers.command {
                        match key {
                            egui::Key::C => self.raw.events.push(Event::Copy),
                            egui::Key::X => self.raw.events.push(Event::Cut),
                            egui::Key::V => {
                                self.raw.events.push(Event::Paste(self.clipboard.clone()))
                            }
                            _ => {}
                        }
                    }
                    self.raw.events.push(Event::Key {
                        key,
                        physical_key: None,
                        pressed: event.state.is_pressed(),
                        repeat: event.repeat,
                        modifiers: self.modifiers,
                    });
                }
                if event.state.is_pressed()
                    && !self.modifiers.command
                    && !self.modifiers.ctrl
                    && let Some(text) = &event.text
                {
                    let text: String = text.chars().filter(|c| !c.is_control()).collect();
                    if !text.is_empty() {
                        self.raw.events.push(Event::Text(text));
                    }
                }
            }
            WindowEvent::Focused(focused) => {
                self.raw.focused = *focused;
                self.raw.events.push(Event::WindowFocused(*focused));
                if !focused {
                    self.touch = None;
                    self.pointer = None;
                }
            }
            _ => {}
        }
    }
    pub fn take_egui_input(&mut self, window: &Window) -> egui::RawInput {
        let scale = window.scale_factor() as f32 * self.ctx.zoom_factor();
        let size = window.inner_size();
        self.raw.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(size.width as f32, size.height as f32) / scale,
        ));
        self.raw.time = Some(self.start.elapsed().as_secs_f64());
        self.raw.take()
    }
    pub fn handle_platform_output(&mut self, window: &Window, output: egui::PlatformOutput) {
        window.set_cursor_visible(output.cursor_icon != egui::CursorIcon::None);
        window.set_cursor(match output.cursor_icon {
            egui::CursorIcon::Text => winit::window::CursorIcon::Text,
            egui::CursorIcon::PointingHand => winit::window::CursorIcon::Pointer,
            egui::CursorIcon::Grab => winit::window::CursorIcon::Grab,
            egui::CursorIcon::Grabbing => winit::window::CursorIcon::Grabbing,
            egui::CursorIcon::ResizeHorizontal => winit::window::CursorIcon::EwResize,
            egui::CursorIcon::ResizeVertical => winit::window::CursorIcon::NsResize,
            _ => winit::window::CursorIcon::Default,
        });
        for command in output.commands {
            match command {
                egui::OutputCommand::CopyText(text) => self.clipboard = text,
                egui::OutputCommand::OpenUrl(url) => {
                    if let Some(browser) = web_sys::window() {
                        let _ = browser.open_with_url_and_target(
                            &url.url,
                            if url.new_tab { "_blank" } else { "_self" },
                        );
                    }
                }
                _ => {}
            }
        }
    }
}
pub fn update_viewport_info(
    info: &mut ViewportInfo,
    _ctx: &egui::Context,
    window: &Window,
    _init: bool,
) {
    info.native_pixels_per_point = Some(window.scale_factor() as f32);
    info.focused = Some(window.has_focus());
}
pub fn process_viewport_commands(
    _ctx: &egui::Context,
    _info: &mut ViewportInfo,
    commands: impl IntoIterator<Item = egui::ViewportCommand>,
    window: &Window,
    _actions: &mut Vec<()>,
) {
    for command in commands {
        if let egui::ViewportCommand::Title(title) = command {
            window.set_title(&title);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn touch_coordinates_scale_and_only_primary_touch_generates_pointer_events() {
        let mut state = State {
            ctx: Default::default(),
            raw: Default::default(),
            modifiers: Default::default(),
            pointer: None,
            touch: None,
            start: wgame::app::time::Instant::now(),
            clipboard: String::new(),
        };
        for (id, phase) in [
            (1, TouchPhase::Started),
            (2, TouchPhase::Started),
            (2, TouchPhase::Ended),
            (1, TouchPhase::Ended),
        ] {
            state.handle(
                &WindowEvent::Touch(winit::event::Touch {
                    device_id: winit::event::DeviceId::dummy(),
                    id,
                    phase,
                    location: (80.0, 60.0).into(),
                    force: None,
                }),
                2.0,
            );
        }
        let buttons: Vec<_> = state
            .raw
            .events
            .iter()
            .filter_map(|event| match event {
                Event::PointerButton { pos, pressed, .. } => Some((*pos, *pressed)),
                _ => None,
            })
            .collect();
        assert_eq!(
            buttons,
            [
                (egui::pos2(40.0, 30.0), true),
                (egui::pos2(40.0, 30.0), false)
            ]
        );
        assert!(matches!(state.raw.events.last(), Some(Event::PointerGone)));
    }
    #[test]
    fn egui_key_names_cover_application_shortcuts() {
        for name in [
            "r", "s", "n", "p", "Space", "Home", "Escape", "=", "-", "\\",
        ] {
            assert!(egui::Key::from_name(name).is_some(), "{name}");
        }
    }
}
