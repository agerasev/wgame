//! Input for a drawing area, independent of a windowing or UI backend.
//!
//! Positions and scrolling use local logical pixels, X right and Y down.
//! Adapters produce one [`CanvasInput`] per application frame. Events retain the
//! order supplied by that adapter, not necessarily the original OS event history.
//! A canvas has one primary pointer (mouse or primary touch), five buttons, and
//! logical keys. Text editing, IME, individual touch IDs and raw device motion
//! belong to the host's platform/UI API.
//!
//! Capture belongs to the canvas where a press started. Movement and release may
//! occur outside its bounds. [`Event::Cancelled`] ends all held interactions on
//! focus loss, geometry changes, or removal; it must not activate a click action.

#![forbid(unsafe_code)]

use glam::Vec2;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Primary,
    Secondary,
    Middle,
    Back,
    Forward,
}
impl Button {
    pub const ALL: [Self; 5] = [
        Self::Primary,
        Self::Secondary,
        Self::Middle,
        Self::Back,
        Self::Forward,
    ];
}

/// Logical key; letters are lowercase, independent of Shift. Use platform events
/// for physical key locations. Text entry should be handled by the UI host.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Character(char),
    Escape,
    Space,
    Enter,
    Tab,
    Backspace,
    Delete,
    Home,
    End,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Plus,
    Minus,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub command: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    Moved(Vec2),
    Button {
        button: Button,
        pressed: bool,
        position: Vec2,
    },
    /// Logical pixels. Positive Y follows the host's scroll direction.
    Scroll(Vec2),
    Key {
        key: Key,
        pressed: bool,
        repeat: bool,
    },
    /// OS window focus; distinct from this canvas's keyboard focus.
    Focused(bool),
    Cancelled,
}

/// Immutable input paired with one content frame.
#[derive(Clone, Debug, Default)]
pub struct CanvasInput {
    pub events: Vec<Event>,
    pub pointer: Option<Vec2>,
    pub hovered: bool,
    pub focused: bool,
    pub window_focused: bool,
    pub modifiers: Modifiers,
    buttons: [bool; 5],
    keys: BTreeSet<Key>,
}
impl CanvasInput {
    pub fn button_down(&self, button: Button) -> bool {
        self.buttons[button as usize]
    }
    pub fn key_down(&self, key: Key) -> bool {
        self.keys.contains(&key)
    }
}

/// Adapter-side accumulator. Clearing a frame keeps held state; cancellation
/// clears held state and emits an explicit cancellation instead of fake releases.
#[derive(Default)]
pub struct InputState {
    input: CanvasInput,
}
impl InputState {
    pub fn input(&self) -> &CanvasInput {
        &self.input
    }
    pub fn push(&mut self, event: Event) {
        match event {
            Event::Moved(pos) => self.input.pointer = Some(pos),
            Event::Button {
                button,
                pressed,
                position,
            } => {
                self.input.pointer = Some(position);
                self.input.buttons[button as usize] = pressed;
            }
            Event::Key { key, pressed, .. } => {
                if pressed {
                    self.input.keys.insert(key);
                } else {
                    self.input.keys.remove(&key);
                }
            }
            Event::Focused(value) => {
                self.input.window_focused = value;
                if !value {
                    self.push(Event::Cancelled);
                }
            }
            Event::Cancelled => {
                self.input.buttons.fill(false);
                self.input.keys.clear();
                self.input.pointer = None;
            }
            Event::Scroll(_) => {}
        }
        self.input.events.push(event);
    }
    pub fn finish(&mut self, hovered: bool, focused: bool, modifiers: Modifiers) -> CanvasInput {
        self.input.hovered = hovered;
        self.input.focused = focused;
        self.input.modifiers = modifiers;
        let result = self.input.clone();
        self.input.events.clear();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn held_state_survives_frames_but_cancellation_never_activates_release() {
        let mut state = InputState::default();
        state.push(Event::Button {
            button: Button::Primary,
            pressed: true,
            position: Vec2::ONE,
        });
        state.push(Event::Key {
            key: Key::Space,
            pressed: true,
            repeat: false,
        });
        assert_eq!(
            state.finish(true, true, Modifiers::default()).events.len(),
            2
        );
        let frame = state.finish(false, true, Modifiers::default());
        assert!(frame.button_down(Button::Primary) && frame.key_down(Key::Space));
        assert!(frame.events.is_empty());
        state.push(Event::Focused(false));
        let frame = state.finish(false, false, Modifiers::default());
        assert!(!frame.button_down(Button::Primary) && !frame.key_down(Key::Space));
        assert_eq!(frame.events, [Event::Cancelled, Event::Focused(false)]);
    }
    #[test]
    fn press_and_release_within_one_frame_are_both_retained() {
        let mut state = InputState::default();
        for pressed in [true, false] {
            state.push(Event::Button {
                button: Button::Primary,
                pressed,
                position: Vec2::ZERO,
            });
        }
        let frame = state.finish(true, true, Modifiers::default());
        assert_eq!(frame.events.len(), 2);
        assert!(!frame.button_down(Button::Primary));
    }
}
