//! On-demand drawing: input/OS redraws, an optional one-second timer, and animation.
//! Assets are embedded. Space toggles the timer; A toggles animation; Esc closes.
#![forbid(unsafe_code)]

use std::{collections::VecDeque, time::Duration};
use wgame::{
    Library, Result, Window,
    app::time::Instant,
    canvas::{Button, Event, Key},
    gfx::types::{Color, color},
    glam::{Vec2, Vec4},
    prelude::*,
};
use wgame_examples::{Labels, gallery};

#[wgame::window(title = "wgame: on-demand events", logical_size = (1200.0, 900.0), resizable = true)]
async fn main(mut window: Window<'_>) -> Result<()> {
    let library = Library::new(window.graphics());
    let shapes = library.shapes();
    let mut labels = Labels::new(&library)?;
    let mut timer = None;
    let mut ticks = 0;
    let mut frames = 0;
    let mut animate = false;
    let mut phase = 0.0_f32;
    let mut last = Instant::now();
    let mut history = VecDeque::new();
    let smoke = cfg!(not(target_arch = "wasm32")) && std::env::args().any(|arg| arg == "--smoke");
    loop {
        // The first frame is unconditional. OS redraw/resize/scale/close and input
        // wake the host independently of this application deadline.
        if frames > 0 {
            let delay = if animate || smoke {
                Some(Duration::ZERO)
            } else {
                timer.map(|deadline: Instant| deadline.saturating_duration_since(Instant::now()))
            };
            window.wait_for_update(delay).await;
        }
        let Some(mut frame) = window.next_frame().await? else {
            break;
        };
        let now = Instant::now();
        if animate {
            phase += (now - last).as_secs_f32().min(0.1);
        }
        last = now;
        let mut close = false;
        for event in &frame.input().events {
            if let Event::Key {
                key,
                pressed: true,
                repeat: false,
            } = event
            {
                match key {
                    Key::Space => {
                        timer = if timer.is_some() {
                            None
                        } else {
                            Some(now + Duration::from_secs(1))
                        };
                    }
                    Key::Character('a') => animate = !animate,
                    Key::Escape => close = true,
                    _ => {}
                }
            }
            let entry = match event {
                Event::Key { key, pressed, .. } => Some(format!(
                    "Key {key:?}: {}",
                    if *pressed { "down" } else { "up" }
                )),
                Event::Button {
                    button, pressed, ..
                } => Some(format!(
                    "{button:?} button: {}",
                    if *pressed { "down" } else { "up" }
                )),
                Event::Scroll(delta) => Some(format!("Scroll: {:.0}, {:.0}", delta.x, delta.y)),
                Event::Focused(focused) => Some(format!("Window focus: {focused}")),
                Event::Cancelled => Some("Held input cancelled".to_owned()),
                Event::Moved(_) => None,
            };
            if let Some(entry) = entry {
                history.push_front(entry);
            }
        }
        if close {
            frame.discard();
            break;
        }
        if let Some((w, h)) = frame.resized() {
            history.push_front(format!("Geometry: {w} x {h} physical pixels"));
        }
        history.truncate(5);
        // An absolute deadline prevents frequent pointer events from postponing
        // the tick. One wake catches up after a long suspension without a burst.
        if timer.is_some_and(|deadline| now >= deadline) {
            ticks += 1;
            timer = Some(now + Duration::from_secs(1));
        }
        let (width, height) = frame.logical_size();
        let dpi = frame.scale_factor();
        let pointer = frame.input().pointer;
        let held = Button::ALL.map(|button| frame.input().button_down(button));
        let modifiers = frame.input().modifiers;
        let (camera, scale) = gallery::camera(&mut frame);
        labels.set_scale(scale);
        frame.clear(gallery::BACKGROUND);
        let mut scene = frame.scene();
        scene.camera = camera;
        gallery::heading(
            &mut scene,
            &mut labels,
            "On-demand events",
            "Space: toggle one-second timer    A: toggle animation    Esc: close",
        );
        for (i, (title, detail)) in [
            (
                "Pointer movement",
                "The marker maps the whole canvas into this rectangle",
            ),
            (
                "Buttons and modifiers",
                "Hold a button or modifier to change its indicator",
            ),
            (
                "Timer wakeups",
                "One step per second, without continuous rendering",
            ),
            (
                "Continuous animation",
                "Only advances while animation is enabled",
            ),
            (
                "Recent events",
                "Keys, buttons, scrolling, focus, and geometry changes",
            ),
            (
                "Repaint state",
                "With both modes off, the frame count settles to idle",
            ),
        ]
        .iter()
        .enumerate()
        {
            gallery::panel(&mut scene, &library, &mut labels, i, title, detail);
        }
        let map_origin = Vec2::new(48.0, 188.0);
        let map_size = Vec2::new(504.0, 99.0);
        scene.add(
            &shapes
                .rectangle((map_origin, map_origin + map_size))
                .fill_color(gallery::BACKGROUND),
        );
        if let Some(p) = pointer {
            let normalized = p / Vec2::new(width as f32, height as f32);
            let mapped = map_origin + normalized.clamp(Vec2::ZERO, Vec2::ONE) * map_size;
            scene.add(
                &shapes
                    .line(
                        Vec2::new(mapped.x, map_origin.y),
                        Vec2::new(mapped.x, map_origin.y + map_size.y),
                        1.0,
                    )
                    .fill_color(color::CYAN),
            );
            scene.add(
                &shapes
                    .line(
                        Vec2::new(map_origin.x, mapped.y),
                        Vec2::new(map_origin.x + map_size.x, mapped.y),
                        1.0,
                    )
                    .fill_color(color::CYAN),
            );
            scene.add(
                &shapes
                    .unit_circle()
                    .stroke_color(0.2, color::YELLOW)
                    .scale(8.0)
                    .move_to(mapped),
            );
        }
        let pointer_label = pointer
            .map(|p| format!("Logical position: {:.0}, {:.0}", p.x, p.y))
            .unwrap_or_else(|| "Pointer outside / cancelled".to_owned());
        scene.add(
            &labels
                .text(&pointer_label, 17.0)
                .move_to(Vec2::new(48.0, 318.0)),
        );
        for (i, name) in ["Left", "Right", "Middle", "Back", "Forward"]
            .into_iter()
            .enumerate()
        {
            let pos = Vec2::new(692.0 + i as f32 * 98.0, 228.0);
            scene.add(
                &shapes
                    .unit_circle()
                    .fill_color(if held[i] {
                        color::CYAN.to_vec4()
                    } else {
                        Vec4::new(0.18, 0.23, 0.3, 1.0)
                    })
                    .scale(21.0)
                    .move_to(pos),
            );
            scene.add(
                &labels
                    .text(name, 15.0)
                    .move_to(pos + Vec2::new(-24.0, 42.0)),
            );
        }
        for (i, (name, on)) in [
            ("Shift", modifiers.shift),
            ("Ctrl", modifiers.control),
            ("Alt", modifiers.alt),
            ("Command", modifiers.command),
        ]
        .into_iter()
        .enumerate()
        {
            scene.add(
                &labels
                    .text(name, 17.0)
                    .multiply_color(if on {
                        color::YELLOW.to_vec4()
                    } else {
                        Vec4::new(0.5, 0.56, 0.64, 1.0)
                    })
                    .move_to(Vec2::new(656.0 + i as f32 * 121.0, 318.0)),
            );
        }
        for i in 0..10 {
            let pos = Vec2::new(70.0 + i as f32 * 51.0, 483.0);
            scene.add(
                &shapes
                    .unit_circle()
                    .fill_color(if i == ticks % 10 {
                        color::YELLOW.to_vec4()
                    } else {
                        Vec4::new(0.18, 0.23, 0.3, 1.0)
                    })
                    .scale(17.0)
                    .move_to(pos),
            );
        }
        scene.add(
            &labels
                .text(&format!("{} ticks", ticks), 32.0)
                .move_to(Vec2::new(48.0, 548.0)),
        );
        scene.add(
            &labels
                .text(
                    if timer.is_some() {
                        "Timer on"
                    } else {
                        "Timer off - press Space"
                    },
                    17.0,
                )
                .move_to(Vec2::new(48.0, 579.0)),
        );
        let center = Vec2::new(905.0, 509.0);
        scene.add(
            &shapes
                .unit_circle()
                .stroke_color(0.025, Vec4::new(0.3, 0.4, 0.5, 1.0))
                .scale(55.0)
                .move_to(center),
        );
        for i in 0..12 {
            let a = phase * 2.0 - i as f32 * 0.12;
            scene.add(
                &shapes
                    .unit_circle()
                    .fill_color(Vec4::new(0.15, 0.8, 1.0, 1.0 - i as f32 / 13.0))
                    .scale(9.0)
                    .move_to(center + Vec2::new(a.cos(), a.sin()) * 55.0),
            );
        }
        scene.add(
            &labels
                .text(
                    if animate {
                        "Animation on"
                    } else {
                        "Animation off - press A"
                    },
                    17.0,
                )
                .move_to(Vec2::new(648.0, 579.0)),
        );
        for (i, entry) in history.iter().enumerate() {
            scene.add(
                &labels
                    .text(entry, 17.0)
                    .move_to(Vec2::new(48.0, 713.0 + i as f32 * 29.0)),
            );
        }
        scene.add(
            &labels
                .text(&format!("Frame {}", frames + 1), 36.0)
                .move_to(Vec2::new(648.0, 726.0)),
        );
        scene.add(
            &labels
                .text(
                    &format!("Canvas: {width:.0} x {height:.0} logical px"),
                    18.0,
                )
                .move_to(Vec2::new(648.0, 769.0)),
        );
        scene.add(
            &labels
                .text(
                    &format!("Display scale: {dpi:.2}  |  Gallery scale: {scale:.2}"),
                    17.0,
                )
                .move_to(Vec2::new(648.0, 800.0)),
        );
        scene.add(
            &labels
                .text(
                    if animate {
                        "Waiting: next frame"
                    } else if timer.is_some() {
                        "Waiting: input or timer deadline"
                    } else {
                        "Waiting: input / OS redraw only"
                    },
                    17.0,
                )
                .move_to(Vec2::new(648.0, 831.0)),
        );
        scene.render();
        frame.present();
        frames += 1;
        if smoke && frames == 12 {
            break;
        }
    }
    Ok(())
}
