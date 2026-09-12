//! Reproducible CPU submission/end-to-end baseline. Uses a headless GPU.
#[path = "../tests/support/mod.rs"]
mod support;
use std::{
    hint::black_box,
    time::{Duration, Instant},
};
use wgame::{
    Library,
    gfx::types::color,
    gfx::{Offscreen, Scene},
    glam::Vec2,
    prelude::*,
};
fn main() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let mut target = Offscreen::new(&gfx, (256, 256));
    let camera = target.physical_camera();
    let circle = lib
        .shapes()
        .unit_circle()
        .fill_color(color::CYAN)
        .scale(3.0);
    let quad = lib
        .shapes()
        .rectangle((Vec2::ZERO, Vec2::splat(6.0)))
        .fill_color(color::RED);
    let font = lib.make_font(
        &wgame::typography::FontData::new(
            include_bytes!("../../wgame-examples/assets/free-sans-bold.ttf").to_vec(),
            0,
        )
        .unwrap(),
    );
    let raster = font.rasterize(16.0);
    // Prime all changing text glyphs so measured frames exclude cache warmup.
    let _ = raster.text("counter 0123456789");
    println!(
        "workload,mode,objects,batches,render_passes,instance_buffers_per_frame,cpu_us,end_to_end_us"
    );
    for workload in ["identical", "mixed", "transparent", "text"] {
        let count = if workload == "text" { 100 } else { 1000 };
        let build = |frame: usize| {
            let mut scene = Scene::default();
            for i in 0..count {
                let pos = Vec2::new((i % 32) as f32 * 8.0, (i / 32) as f32 * 8.0);
                match workload {
                    "text" => scene.add(
                        &raster
                            .text(&format!("counter {}", frame % 100))
                            .scale(16.0)
                            .move_to(pos),
                    ),
                    "mixed" if i % 2 == 0 => scene.add(&quad.move_to(pos)),
                    "transparent" if i % 2 == 0 => scene.add(
                        &quad
                            .multiply_color(wgame::rgb::Rgba::new(1.0, 1.0, 1.0, 0.5))
                            .move_to(pos * 0.1),
                    ),
                    "transparent" => scene.add(
                        &circle
                            .multiply_color(wgame::rgb::Rgba::new(1.0, 1.0, 1.0, 0.5))
                            .move_to(pos * 0.1),
                    ),
                    _ => scene.add(&circle.move_to(pos)),
                }
            }
            scene
        };
        for mode in ["multipass", "singlepass", "retained"] {
            // Retained is deliberately not used for changing text.
            if workload == "text" && mode == "retained" {
                continue;
            }
            let snapshot = build(0).bake();
            let mut cpu = Duration::ZERO;
            let mut total = Duration::ZERO;
            let batches = snapshot.len();
            for frame in 0..50 {
                let start = Instant::now();
                target.clear(color::BLACK);
                if mode == "retained" {
                    target.render(&camera, &snapshot);
                } else {
                    let scene = build(frame);
                    if mode == "multipass" {
                        for batch in scene.iter() {
                            target.render(&camera, batch);
                        }
                    } else {
                        target.render_iter(&camera, scene.iter());
                    }
                }
                target.submit();
                let cpu_elapsed = start.elapsed();
                gfx.device()
                    .poll(wgpu::PollType::Wait {
                        submission_index: None,
                        timeout: Some(Duration::from_secs(30)),
                    })
                    .unwrap();
                if frame >= 10 {
                    cpu += cpu_elapsed;
                    total += start.elapsed();
                }
            }
            black_box(&target);
            println!(
                "{workload},{mode},{count},{batches},{},{},{:.1},{:.1}",
                if mode == "multipass" { batches + 1 } else { 2 },
                if mode == "retained" { 0 } else { batches },
                cpu.as_secs_f64() * 1e6 / 40.0,
                total.as_secs_f64() * 1e6 / 40.0
            );
        }
    }
}
