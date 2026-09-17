use egui::{PointerButton, Response};
use wgame::canvas::{Button, CanvasInput, Event, InputState, Key, Modifiers};
use wgame::glam::Vec2;

#[derive(Default)]
pub(crate) struct CanvasState {
    state: InputState,
    geometry: Option<(egui::Rect, f32)>,
}
impl CanvasState {
    pub fn collect(
        &mut self,
        response: &Response,
        events: &[egui::Event],
        window_focused: bool,
        modifiers: egui::Modifiers,
        scale: f32,
    ) -> CanvasInput {
        // Egui surrenders widget focus on Escape before returning the response.
        // A canvas owns Escape (e.g. cancel an aim), so keep its focus and deliver
        // the key. Never restore focus stolen by another widget or window.
        if window_focused
            && self.state.input().focused
            && response.ctx.memory(|m| m.focused().is_none())
            && events.iter().any(|event| {
                matches!(
                    event,
                    egui::Event::Key {
                        key: egui::Key::Escape,
                        pressed: true,
                        ..
                    }
                )
            })
        {
            response.request_focus();
        }
        let geometry = (response.rect, scale);
        if self.geometry != Some(geometry) || (self.state.input().focused && !response.has_focus())
        {
            self.state.push(Event::Cancelled);
        }
        self.geometry = Some(geometry);
        if window_focused != self.state.input().window_focused {
            self.state.push(Event::Focused(window_focused));
        }
        let local = |pos: egui::Pos2| {
            let p = pos - response.rect.min;
            Vec2::new(p.x, p.y)
        };
        for event in events {
            let captured = Button::ALL
                .into_iter()
                .any(|b| self.state.input().button_down(b));
            match *event {
                egui::Event::PointerMoved(pos) if response.hovered() || captured => {
                    self.state.push(Event::Moved(local(pos)));
                }
                egui::Event::PointerButton {
                    pos,
                    button,
                    pressed,
                    ..
                } => {
                    let mapped = button_key(button);
                    let owns_press = response.is_pointer_button_down_on()
                        || response.clicked_by(button)
                        || response.drag_started_by(button);
                    if window_focused
                        && ((pressed && owns_press && response.interact_rect.contains(pos))
                            || (!pressed && self.state.input().button_down(mapped)))
                    {
                        self.state.push(Event::Button {
                            button: mapped,
                            pressed,
                            position: local(pos),
                        });
                    }
                }
                egui::Event::WindowFocused(value) => self.state.push(Event::Focused(value)),
                egui::Event::PointerGone => self.state.push(Event::Cancelled),
                egui::Event::MouseWheel { unit, delta, .. } if response.hovered() => {
                    let factor = match unit {
                        egui::MouseWheelUnit::Point => Vec2::ONE,
                        egui::MouseWheelUnit::Line => Vec2::splat(40.0),
                        egui::MouseWheelUnit::Page => {
                            Vec2::new(response.rect.width(), response.rect.height())
                        }
                    };
                    self.state
                        .push(Event::Scroll(Vec2::new(delta.x, delta.y) * factor));
                }
                egui::Event::Key {
                    key,
                    pressed,
                    repeat,
                    ..
                } if response.has_focus() && window_focused => {
                    if let Some(key) = logical_key(key) {
                        self.state.push(Event::Key {
                            key,
                            pressed,
                            repeat,
                        });
                    }
                }
                _ => {}
            }
        }
        self.state.finish(
            response.hovered(),
            response.has_focus(),
            Modifiers {
                shift: modifiers.shift,
                control: modifiers.ctrl,
                alt: modifiers.alt,
                command: modifiers.mac_cmd,
            },
        )
    }
}
fn button_key(button: PointerButton) -> Button {
    match button {
        PointerButton::Primary => Button::Primary,
        PointerButton::Secondary => Button::Secondary,
        PointerButton::Middle => Button::Middle,
        PointerButton::Extra1 => Button::Back,
        PointerButton::Extra2 => Button::Forward,
    }
}
fn logical_key(key: egui::Key) -> Option<Key> {
    Some(match key {
        egui::Key::Escape => Key::Escape,
        egui::Key::Space => Key::Space,
        egui::Key::Enter => Key::Enter,
        egui::Key::Tab => Key::Tab,
        egui::Key::Backspace => Key::Backspace,
        egui::Key::Delete => Key::Delete,
        egui::Key::Home => Key::Home,
        egui::Key::End => Key::End,
        egui::Key::PageUp => Key::PageUp,
        egui::Key::PageDown => Key::PageDown,
        egui::Key::ArrowLeft => Key::ArrowLeft,
        egui::Key::ArrowRight => Key::ArrowRight,
        egui::Key::ArrowUp => Key::ArrowUp,
        egui::Key::ArrowDown => Key::ArrowDown,
        egui::Key::Plus | egui::Key::Equals => Key::Plus,
        egui::Key::Minus => Key::Minus,
        egui::Key::Backslash => Key::Character('\\'),
        other => {
            let name = other.name();
            let mut chars = name.chars();
            let ch = chars.next()?;
            if chars.next().is_some() {
                return None;
            }
            Key::Character(ch.to_ascii_lowercase())
        }
    })
}

#[cfg(test)]
#[path = "input_tests.rs"]
mod tests;
