use super::*;
use crate::Canvas;

struct Harness {
    ctx: egui::Context,
    state: CanvasState,
    text: String,
}
impl Harness {
    fn new() -> Self {
        let mut this = Self {
            ctx: Default::default(),
            state: Default::default(),
            text: String::new(),
        };
        this.frame(Vec::new(), 1.0, true, 100.0);
        this.frame(Vec::new(), 1.0, true, 100.0);
        this
    }
    fn frame(
        &mut self,
        events: Vec<egui::Event>,
        scale: f32,
        focused: bool,
        panel: f32,
    ) -> CanvasInput {
        let mut response = None;
        let mut raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 300.0),
            )),
            events: events.clone(),
            focused,
            ..Default::default()
        };
        raw.viewports
            .get_mut(&egui::ViewportId::ROOT)
            .unwrap()
            .native_pixels_per_point = Some(scale);
        let output = self.ctx.run_ui(raw, |ui| {
            egui::Panel::left("controls")
                .exact_size(panel)
                .show(ui, |ui| {
                    ui.text_edit_singleline(&mut self.text);
                    let _ = ui.button("button");
                });
            response = Some(
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show(ui, |ui| {
                        Canvas {
                            texture: egui::TextureId::User(0),
                        }
                        .show(ui)
                    })
                    .inner,
            );
        });
        output.drop_without_applying_deltas();
        self.state.collect(
            &response.unwrap(),
            &events,
            focused,
            self.ctx.input(|i| i.modifiers),
            scale,
        )
    }
}
fn mouse(x: f32, y: f32, pressed: bool) -> Vec<egui::Event> {
    vec![
        egui::Event::PointerMoved(egui::pos2(x, y)),
        egui::Event::PointerButton {
            pos: egui::pos2(x, y),
            button: PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        },
    ]
}
fn key(pressed: bool) -> egui::Event {
    egui::Event::Key {
        key: egui::Key::Space,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: Default::default(),
    }
}
#[test]
fn ui_press_is_excluded_and_canvas_capture_releases_outside() {
    let mut h = Harness::new();
    let input = h.frame(mouse(30.0, 40.0, true), 1.0, true, 100.0);
    assert!(!input.button_down(Button::Primary));
    h.frame(mouse(30.0, 40.0, false), 1.0, true, 100.0);
    let input = h.frame(mouse(200.0, 150.0, true), 1.0, true, 100.0);
    assert!(input.button_down(Button::Primary), "{:?}", input.events);
    let input = h.frame(mouse(30.0, 150.0, false), 1.0, true, 100.0);
    assert!(!input.button_down(Button::Primary));
    assert!(
        input.events.iter().any(
            |e| matches!(e, Event::Button { pressed:false, position, .. } if position.x < 0.0)
        )
    );
}
#[test]
fn text_focus_excludes_game_keys_and_focus_loss_cancels_drag() {
    let mut h = Harness::new();
    h.frame(mouse(200.0, 150.0, true), 1.0, true, 100.0);
    h.frame(mouse(200.0, 150.0, false), 1.0, true, 100.0);
    assert!(
        h.frame(vec![key(true)], 1.0, true, 100.0)
            .key_down(Key::Space)
    );
    h.frame(mouse(30.0, 10.0, true), 1.0, true, 100.0);
    h.frame(mouse(30.0, 10.0, false), 1.0, true, 100.0);
    let input = h.frame(vec![key(true)], 1.0, true, 100.0);
    assert!(!input.key_down(Key::Space));
    h.frame(mouse(200.0, 150.0, true), 1.0, true, 100.0);
    let input = h.frame(Vec::new(), 1.0, false, 100.0);
    assert!(!input.button_down(Button::Primary));
    assert!(input.events.contains(&Event::Cancelled));
}
#[test]
fn resize_and_dpi_change_cancel_without_fake_release() {
    for (scale, panel) in [(1.0, 120.0), (2.0, 100.0)] {
        let mut h = Harness::new();
        h.frame(mouse(200.0, 150.0, true), 1.0, true, 100.0);
        let input = h.frame(Vec::new(), scale, true, panel);
        assert!(!input.button_down(Button::Primary));
        assert!(input.events.contains(&Event::Cancelled));
        assert!(
            !input
                .events
                .iter()
                .any(|e| matches!(e, Event::Button { pressed: false, .. }))
        );
    }
}
#[test]
fn quick_click_keeps_press_and_release_in_order() {
    let mut h = Harness::new();
    let mut events = mouse(200.0, 150.0, true);
    events.extend(mouse(200.0, 150.0, false));
    let input = h.frame(events, 1.0, true, 100.0);
    let pressed: Vec<_> = input
        .events
        .iter()
        .filter_map(|e| match e {
            Event::Button { pressed, .. } => Some(*pressed),
            _ => None,
        })
        .collect();
    assert_eq!(pressed, [true, false]);
}

#[test]
fn escape_is_delivered_to_focused_canvas_without_cancelling_first() {
    let mut h = Harness::new();
    h.frame(mouse(200.0, 150.0, true), 1.0, true, 100.0);
    h.frame(mouse(200.0, 150.0, false), 1.0, true, 100.0);
    h.frame(mouse(200.0, 150.0, true), 1.0, true, 100.0);
    let input = h.frame(
        vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
        1.0,
        true,
        100.0,
    );
    assert!(
        input.events.iter().any(|e| matches!(
            e,
            Event::Key {
                key: Key::Escape,
                pressed: true,
                ..
            }
        )),
        "{:?}",
        input.events
    );
    assert!(!input.events.contains(&Event::Cancelled));
    assert!(input.button_down(Button::Primary));
}
