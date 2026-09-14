//! Same content loop with a plain window (`--plain`) or egui controls.
use wgame::{ContentFrame, WindowHost, gfx::types::color, glam::Vec2, prelude::*};
use wgame_egui::{EguiWindow, egui};

#[wgame::window(title = "wgame egui", logical_size = (800.0, 600.0), resizable = true)]
async fn main(window: wgame::Window<'_>) -> wgame::Result<()> {
    if std::env::args().any(|arg| arg == "--plain") {
        run(window).await
    } else {
        let mut text = String::from("Keyboard input stays here");
        run(EguiWindow::new(window, move |ui, canvas| {
            egui::Panel::left("controls").show(ui, |ui| {
                ui.heading("wgame + egui");
                ui.text_edit_singleline(&mut text);
                ui.label("Click the canvas to focus it.\nDrag the ring with the pointer.");
            });
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ui, |ui| canvas.show(ui))
                .inner
        }))
        .await
    }
}
async fn run(mut host: impl WindowHost) -> wgame::Result<()> {
    let library = wgame::Library::new(host.graphics());
    let smoke = std::env::args().any(|arg| arg == "--smoke");
    let mut count = 0;
    while let Some(mut frame) = host.next_frame().await? {
        let pointer = frame.input().pointer.unwrap_or(Vec2::splat(100.0));
        frame.clear(color::BLACK);
        let camera = frame.logical_camera();
        let mut scene = frame.scene();
        scene.camera = camera;
        scene.add(
            &library
                .shapes()
                .unit_circle()
                .scale(30.0)
                .move_to(pointer)
                .fill_color(color::RED),
        );
        scene.render();
        count += 1;
        // Exercise the explicit discard path without preventing subsequent frames.
        if count == 2 {
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
