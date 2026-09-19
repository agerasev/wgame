//! One event-driven content loop with a plain window (`--plain`) or egui controls.
//! Space toggles animation when the canvas is focused. `--smoke` checks lifecycle,
//! then (with egui) verifies idle waiting and delayed/immediate repaint requests.
use std::{cell::Cell, rc::Rc, time::Duration};
use wgame::{
    ContentFrame, WindowHost,
    app::time::Instant,
    canvas::{Event, Key},
    gfx::types::color,
    glam::Vec2,
    prelude::*,
};
use wgame_egui::{EguiWindow, egui};
#[path = "support/repaint.rs"]
mod repaint;

#[wgame::window(title = "wgame egui: on-demand drawing", logical_size = (800.0, 600.0), resizable = true)]
async fn main(mut window: wgame::Window<'_>) -> wgame::Result<()> {
    let animate = Rc::new(Cell::new(false));
    if flag("--plain") {
        run(&mut window, &animate).await
    } else {
        let mut text = String::from("Keyboard input stays here");
        let mut deadline = None;
        let mut host = EguiWindow::new(window, {
            let animate = animate.clone();
            move |ui, canvas| {
                egui::Panel::left("controls").show(ui, |ui| {
                    ui.heading("wgame + egui");
                    ui.text_edit_singleline(&mut text);
                    let mut active = animate.get();
                    if ui.checkbox(&mut active, "Animate ring").changed() { animate.set(active); }
                    if ui.button("Repaint after one second").on_hover_text("Tooltips also wake an idle window at egui's deadline.").clicked() {
                        deadline = Some(Instant::now() + Duration::from_secs(1));
                    }
                    if let Some(at) = deadline {
                        let remaining = at.saturating_duration_since(Instant::now());
                        if remaining.is_zero() { ui.label("Delayed repaint completed"); }
                        else {
                            ui.label("Waiting for repaint...");
                            ui.ctx().request_repaint_after(remaining);
                        }
                    }
                    ui.label("Move the ring with the pointer.\nClick the canvas; Space toggles animation.\nResize while idle to update the canvas.");
                });
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show(ui, |ui| canvas.show(ui))
                    .inner
            }
        });
        run(&mut host, &animate).await?;
        if flag("--smoke") {
            repaint::smoke(&mut host).await?;
        }
        Ok(())
    }
}
async fn run(host: &mut impl WindowHost, animate: &Cell<bool>) -> wgame::Result<()> {
    let library = wgame::Library::new(host.graphics());
    let smoke = flag("--smoke");
    let mut count = 0;
    let mut phase = 0.0_f32;
    let mut last = Instant::now();
    let mut followup = false;
    loop {
        if count > 0 {
            host.wait_for_update((smoke || animate.get() || followup).then_some(Duration::ZERO))
                .await;
        }
        let Some(mut frame) = host.next_frame().await? else {
            break;
        };
        let now = Instant::now();
        if animate.get() {
            phase += (now - last).as_secs_f32().min(0.1);
        }
        last = now;
        followup = false;
        for event in &frame.input().events {
            if matches!(
                event,
                Event::Key {
                    key: Key::Space,
                    pressed: true,
                    repeat: false
                }
            ) {
                animate.set(!animate.get());
                // Controls were laid out before this keyboard event was applied.
                followup = true;
            }
        }
        let pointer = frame.input().pointer.unwrap_or(Vec2::splat(100.0));
        frame.clear(color::BLACK);
        let camera = frame.logical_camera();
        let mut scene = frame.scene();
        scene.camera = camera;
        scene.add(
            &library
                .shapes()
                .unit_circle()
                .scale(30.0 + 8.0 * phase.sin())
                .move_to(pointer)
                .fill_color(color::RED),
        );
        scene.render();
        count += 1;
        // Exercise discard in smoke mode only; it requires another frame.
        if smoke && count == 2 {
            frame.discard();
        } else {
            frame.present();
        }
        if smoke && count == 12 {
            break;
        }
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn flag(name: &str) -> bool {
    std::env::args().any(|arg| arg == name)
}
#[cfg(target_arch = "wasm32")]
fn flag(_: &str) -> bool {
    false
}
