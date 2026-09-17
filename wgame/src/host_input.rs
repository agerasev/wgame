use crate::app::{
    Input,
    input::{
        Event,
        event::{MouseButton, MouseScrollDelta, TouchPhase},
        keyboard::{Key as WinitKey, NamedKey},
    },
};
use crate::canvas::{Button, CanvasInput, Event as CanvasEvent, InputState, Key, Modifiers};
use glam::Vec2;

pub(crate) struct PlainInput {
    events: Input,
    state: InputState,
    pixel: Option<Vec2>,
    touch: Option<u64>,
    modifiers: Modifiers,
}
impl PlainInput {
    pub fn new(events: Input, focused: bool) -> Self {
        let mut state = InputState::default();
        state.push(CanvasEvent::Focused(focused));
        Self {
            events,
            state,
            pixel: None,
            touch: None,
            modifiers: Modifiers::default(),
        }
    }
    pub fn collect(&mut self, scale: f64, changed: bool) -> CanvasInput {
        let scale = scale as f32;
        if changed {
            self.state.push(CanvasEvent::Cancelled);
            self.touch = None;
            if let Some(pixel) = self.pixel {
                self.state.push(CanvasEvent::Moved(pixel / scale));
            }
        }
        while let Some(event) = self.events.try_next() {
            match event {
                Event::CursorMoved { position, .. } if self.touch.is_none() => {
                    let pixel = Vec2::new(position.x as f32, position.y as f32);
                    self.pixel = Some(pixel);
                    self.state.push(CanvasEvent::Moved(pixel / scale));
                }
                Event::MouseInput { state, button, .. } if self.touch.is_none() => {
                    if let (Some(button), Some(pixel)) = (button_key(button), self.pixel) {
                        self.state.push(CanvasEvent::Button {
                            button,
                            pressed: state.is_pressed(),
                            position: pixel / scale,
                        });
                    }
                }
                Event::MouseWheel { delta, .. } => {
                    let delta = match delta {
                        MouseScrollDelta::LineDelta(x, y) => Vec2::new(x, y) * 40.0,
                        MouseScrollDelta::PixelDelta(p) => {
                            Vec2::new(p.x as f32, p.y as f32) / scale
                        }
                    };
                    self.state.push(CanvasEvent::Scroll(delta));
                }
                Event::KeyboardInput { event, .. } => {
                    if let Some(key) = logical_key(&event.logical_key) {
                        self.state.push(CanvasEvent::Key {
                            key,
                            pressed: event.state.is_pressed(),
                            repeat: event.repeat,
                        });
                    }
                }
                Event::ModifiersChanged(modifiers) => {
                    let m = modifiers.state();
                    self.modifiers = Modifiers {
                        shift: m.shift_key(),
                        control: m.control_key(),
                        alt: m.alt_key(),
                        command: m.super_key(),
                    };
                }
                Event::Touch(touch) => {
                    let position =
                        Vec2::new(touch.location.x as f32, touch.location.y as f32) / scale;
                    match touch.phase {
                        TouchPhase::Started if self.touch.is_none() => {
                            self.touch = Some(touch.id);
                            self.state.push(CanvasEvent::Button {
                                button: Button::Primary,
                                pressed: true,
                                position,
                            });
                        }
                        TouchPhase::Moved if self.touch == Some(touch.id) => {
                            self.state.push(CanvasEvent::Moved(position))
                        }
                        TouchPhase::Ended if self.touch == Some(touch.id) => {
                            self.state.push(CanvasEvent::Button {
                                button: Button::Primary,
                                pressed: false,
                                position,
                            });
                            self.touch = None;
                        }
                        TouchPhase::Cancelled if self.touch == Some(touch.id) => {
                            self.state.push(CanvasEvent::Cancelled);
                            self.touch = None;
                        }
                        _ => {}
                    }
                }
                Event::Focused(value) => {
                    self.state.push(CanvasEvent::Focused(value));
                    if !value {
                        self.touch = None;
                        self.pixel = None;
                    }
                }
                Event::CursorLeft { .. } if self.touch.is_none() => {
                    self.pixel = None;
                    self.state.push(CanvasEvent::Cancelled);
                }
                _ => {}
            }
        }
        let focused = self.state.input().window_focused;
        self.state
            .finish(self.pixel.is_some(), focused, self.modifiers)
    }
}
fn button_key(button: MouseButton) -> Option<Button> {
    Some(match button {
        MouseButton::Left => Button::Primary,
        MouseButton::Right => Button::Secondary,
        MouseButton::Middle => Button::Middle,
        MouseButton::Back => Button::Back,
        MouseButton::Forward => Button::Forward,
        _ => return None,
    })
}
fn logical_key(key: &WinitKey) -> Option<Key> {
    Some(match key {
        WinitKey::Character(s) => {
            let mut chars = s.chars();
            let ch = chars.next()?;
            if chars.next().is_some() {
                return None;
            }
            match ch {
                '+' | '=' => Key::Plus,
                '-' => Key::Minus,
                _ => Key::Character(ch.to_ascii_lowercase()),
            }
        }
        WinitKey::Named(name) => match name {
            NamedKey::Escape => Key::Escape,
            NamedKey::Space => Key::Space,
            NamedKey::Enter => Key::Enter,
            NamedKey::Tab => Key::Tab,
            NamedKey::Backspace => Key::Backspace,
            NamedKey::Delete => Key::Delete,
            NamedKey::Home => Key::Home,
            NamedKey::End => Key::End,
            NamedKey::PageUp => Key::PageUp,
            NamedKey::PageDown => Key::PageDown,
            NamedKey::ArrowLeft => Key::ArrowLeft,
            NamedKey::ArrowRight => Key::ArrowRight,
            NamedKey::ArrowUp => Key::ArrowUp,
            NamedKey::ArrowDown => Key::ArrowDown,
            _ => return None,
        },
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::input::event::{DeviceId, ElementState};
    use wgame_app_input::EventHandler;
    #[test]
    fn page_keys_reach_the_canvas() {
        for (source, expected) in [
            (NamedKey::PageUp, Key::PageUp),
            (NamedKey::PageDown, Key::PageDown),
        ] {
            assert_eq!(logical_key(&WinitKey::Named(source)), Some(expected));
        }
    }
    #[test]
    fn local_coordinates_and_scale_change_reset_capture() {
        let mut handler = EventHandler::default();
        let mut input = PlainInput::new(handler.input(), true);
        handler.push(Event::CursorMoved {
            device_id: DeviceId::dummy(),
            position: (80.0, 60.0).into(),
        });
        handler.push(Event::MouseInput {
            device_id: DeviceId::dummy(),
            state: ElementState::Pressed,
            button: MouseButton::Left,
        });
        let frame = input.collect(2.0, false);
        assert_eq!(frame.pointer, Some(Vec2::new(40.0, 30.0)));
        assert!(frame.button_down(Button::Primary));
        let frame = input.collect(1.0, true);
        assert_eq!(frame.pointer, Some(Vec2::new(80.0, 60.0)));
        assert!(!frame.button_down(Button::Primary));
        assert!(frame.events.contains(&CanvasEvent::Cancelled));
    }
    #[test]
    fn secondary_touches_do_not_steal_primary_capture() {
        let mut handler = EventHandler::default();
        let mut input = PlainInput::new(handler.input(), true);
        for (id, phase, pos) in [
            (1, TouchPhase::Started, 20.0),
            (2, TouchPhase::Started, 40.0),
            (2, TouchPhase::Ended, 50.0),
            (1, TouchPhase::Moved, 30.0),
        ] {
            handler.push(Event::Touch(crate::app::input::event::Touch {
                device_id: DeviceId::dummy(),
                phase,
                location: (pos, pos).into(),
                force: None,
                id,
            }));
        }
        let frame = input.collect(2.0, false);
        assert!(frame.button_down(Button::Primary));
        assert_eq!(frame.pointer, Some(Vec2::splat(15.0)));
        handler.push(Event::Focused(false));
        assert!(!input.collect(2.0, false).button_down(Button::Primary));
    }
}
