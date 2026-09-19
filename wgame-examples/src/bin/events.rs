//! On-demand drawing: input/OS redraws, an optional one-second timer, and animation.
//! Assets are embedded. Space toggles the timer; A toggles animation; Esc closes.
#![forbid(unsafe_code)]

use std::time::Duration;
use wgame::{
    Library, Result, Window,
    app::time::Instant,
    canvas::{Event, Key},
    gfx::types::color,
    glam::Vec2,
    prelude::*,
    typography::FontData,
};

#[wgame::window(title = "Events: Space = timer, A = animate, Esc = close", logical_size = (960.0, 640.0), resizable = true)]
async fn main(mut window: Window<'_>) -> Result<()> {
    let library = Library::new(window.graphics());
    let font = library.make_font(&FontData::new(
        include_bytes!("../../assets/free-sans-bold.ttf").to_vec(),
        0,
    )?);
    let mut scale = window.scale_factor();
    let mut raster = font.rasterize(24.0 * scale as f32);
    let mut timer = None;
    let mut ticks = 0;
    let mut frames = 0;
    let mut animate = false;
    let mut phase = 0.0_f32;
    let mut last = Instant::now();
    let mut pointer = Vec2::splat(160.0);
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
                        }
                    }
                    Key::Character('a') => animate = !animate,
                    Key::Escape => close = true,
                    _ => {}
                }
            }
        }
        if close {
            frame.discard();
            break;
        }
        // An absolute deadline prevents frequent pointer events from postponing
        // the tick. One wake catches up after a long suspension without a burst.
        if timer.is_some_and(|deadline| now >= deadline) {
            ticks += 1;
            timer = Some(now + Duration::from_secs(1));
        }
        if let Some(pos) = frame.input().pointer {
            pointer = pos;
        }
        if frame.scale_factor() != scale {
            scale = frame.scale_factor();
            raster = font.rasterize(24.0 * scale as f32);
        }
        let (width, height) = frame.logical_size();
        frame.clear(color::BLACK);
        let camera = frame.logical_camera();
        let mut scene = frame.scene();
        scene.camera = camera;
        scene.add(
            &library
                .shapes()
                .unit_circle()
                .stroke_color(0.15, color::CYAN)
                .scale(30.0 + 8.0 * phase.sin())
                .move_to(pointer),
        );
        for (row, label) in [
            "Move the pointer / resize the window / Esc: close".to_owned(),
            format!(
                "Space: timer {} | A: animation {}",
                if timer.is_some() { "on" } else { "off" },
                if animate { "on" } else { "off" }
            ),
            format!("Frame {} | timer ticks {}", frames + 1, ticks),
            format!("Canvas {width:.0} x {height:.0} | scale {scale:.2}"),
        ]
        .iter()
        .enumerate()
        {
            scene.add(
                &raster
                    .text(label)
                    .scale(24.0)
                    .move_to(Vec2::new(16.0, 32.0 + row as f32 * 32.0)),
            );
        }
        scene.render();
        frame.present();
        frames += 1;
        if smoke && frames == 12 {
            break;
        }
    }
    Ok(())
}
