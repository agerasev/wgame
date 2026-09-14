//! Explicit Mesa/GPU checks: `cargo test -p wgame-egui --lib -- --ignored`.
use super::*;

// The shared readback helper keeps identical adapter requirements and timeouts.
#[path = "../../wgame/tests/support/mod.rs"]
mod support;

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn composition_clips_content_and_rebinds_after_resize() {
    let graphics = support::graphics();
    let mut painter = Painter::new(&graphics);
    let ctx = egui::Context::default();
    for (size, scale) in [((80, 80), 1.0), ((120, 120), 2.0)] {
        painter.resize(size);
        painter.target.clear(wgame::gfx::types::color::RED);
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(128.0, 128.0),
                )),
                ..Default::default()
            },
            |ui| {
                ui.painter()
                    .with_clip_rect(egui::Rect::from_min_max(
                        egui::pos2(32.0, 32.0),
                        egui::pos2(96.0, 96.0),
                    ))
                    .image(
                        painter.canvas.texture,
                        egui::Rect::from_min_max(egui::pos2(16.0, 16.0), egui::pos2(112.0, 112.0)),
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
            },
        );
        for (id, deltas) in output.textures_delta.set.drain() {
            for delta in deltas {
                painter
                    .renderer
                    .update_texture(graphics.device(), graphics.queue(), id, &delta);
            }
        }
        let jobs = ctx.tessellate(output.shapes, scale);
        let side = (128.0 * scale) as u32;
        let mut target = gfx::Offscreen::new(&graphics, (side, side));
        let before = painter.paint(
            &mut target,
            &jobs,
            &egui_wgpu::ScreenDescriptor {
                size_in_pixels: [side, side],
                pixels_per_point: scale,
            },
        );
        graphics.queue().submit(before);
        let pixels = support::pixels(&mut target);
        for (x, y, expected) in [
            (64, 64, [255, 0, 0, 255]),
            (20, 64, [0, 0, 0, 255]),
            (100, 64, [0, 0, 0, 255]),
        ] {
            let offset =
                (((y as f32 * scale) as u32 * side + (x as f32 * scale) as u32) * 4) as usize;
            assert_eq!(
                &pixels[offset..offset + 4],
                &expected,
                "scale={scale}, pixel=({x},{y})"
            );
        }
        for id in output.textures_delta.free.drain() {
            painter.renderer.free_texture(&id);
        }
    }
}
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn discarded_canvas_commands_do_not_leak_into_next_frame() {
    let graphics = support::graphics();
    let mut target = gfx::Offscreen::new(&graphics, (8, 8));
    target.clear(wgame::gfx::types::color::RED);
    target.submit();
    target.clear(wgame::gfx::types::color::BLUE);
    target.discard();
    assert!(
        support::pixels(&mut target)
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [255, 0, 0, 255])
    );
    target.clear(wgame::gfx::types::color::GREEN);
    drop(target.finish());
    assert!(
        support::pixels(&mut target)
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [255, 0, 0, 255])
    );
}
