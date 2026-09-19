//! A gallery of shapes, compositing, transforms, textures, typography, and input.
//! The scoped background task is cancelled on shutdown. --smoke draws 12 frames.
use std::{cell::Cell, rc::Rc, time::Duration};
use wgame::{
    Library, Result, Window,
    canvas::{Button, Event, Key},
    gfx::types::{Color, color},
    glam::{Affine2, Vec2, Vec4},
    image::Image,
    prelude::*,
    texture::TextureSettings,
    typography::TextAlign,
};
use wgame_examples::{Labels, gallery};

#[wgame::window(title = "wgame: drawing gallery", logical_size = (1200.0, 900.0))]
async fn main(mut window: Window<'_>) -> Result<()> {
    let library = Library::new(window.graphics());
    let shapes = library.shapes();
    let mut labels = Labels::new(&library)?;
    let image = Image::decode_auto(include_bytes!("../../assets/lenna.png"))?;
    let photo = library.make_texture(&image, TextureSettings::linear());
    let pixels = Image::with_data(
        (8, 8),
        (0..64)
            .map(|i| {
                if (i / 8 + i % 8) % 2 == 0 {
                    color::CYAN
                } else {
                    color::BLUE
                }
                .to_rgba_f16()
            })
            .collect::<Vec<_>>(),
    );
    let nearest = library.make_texture(&pixels, TextureSettings::nearest());
    let linear = library.make_texture(&pixels, TextureSettings::linear());
    let gradient = library
        .texturing()
        .gradient2([[color::CYAN, color::MAGENTA], [color::BLUE, color::YELLOW]]);
    let ticks = Rc::new(Cell::new(0usize));
    let counter = ticks.clone();
    let background = wgame::spawn(async move {
        loop {
            wgame::sleep(Duration::from_millis(250)).await;
            counter.set(counter.get() + 1);
        }
    })
    .cancel_on_drop();
    let mut paused = false;
    let mut angle = 0.0;
    let mut last = wgame::app::time::Instant::now();
    let mut stamps = Vec::new();
    let smoke = cfg!(not(target_arch = "wasm32")) && std::env::args().any(|arg| arg == "--smoke");
    let mut frames = 0;
    'frames: while let Some(mut frame) = window.next_frame().await? {
        let (camera, scale) = gallery::camera(&mut frame);
        labels.set_scale(scale);
        let dpi = frame.scale_factor() as f32;
        let input_origin = gallery::origin(5) + Vec2::new(18.0, 110.0);
        let input_end = gallery::origin(5) + Vec2::new(522.0, 212.0);
        let inside = |p: Vec2| p.cmpge(input_origin).all() && p.cmple(input_end).all();
        for event in &frame.input().events {
            match *event {
                Event::Key {
                    key: Key::Space,
                    pressed: true,
                    repeat: false,
                } => paused = !paused,
                Event::Key {
                    key: Key::Character('r'),
                    pressed: true,
                    ..
                } => stamps.clear(),
                Event::Key {
                    key: Key::Escape,
                    pressed: true,
                    ..
                } => {
                    frame.discard();
                    break 'frames;
                }
                Event::Button {
                    button: Button::Primary,
                    pressed: true,
                    position,
                } => {
                    if let Some(p) = camera
                        .screen_to_world(position * dpi, frame.size())
                        .filter(|p| inside(*p))
                    {
                        if stamps.len() == 16 {
                            stamps.remove(0);
                        }
                        stamps.push(p);
                    }
                }
                _ => (),
            }
        }
        let now = wgame::app::time::Instant::now();
        if !paused {
            angle += (now - last).as_secs_f32().min(0.1);
        }
        last = now;
        let pointer = frame
            .input()
            .pointer
            .filter(|_| frame.input().hovered)
            .and_then(|p| camera.screen_to_world(p * dpi, frame.size()));
        frame.clear(gallery::BACKGROUND);
        let mut scene = frame.scene();
        scene.camera = camera;
        gallery::heading(
            &mut scene,
            &mut labels,
            "Drawing gallery",
            "Space: pause / resume    R: clear stamps    Esc: close",
        );
        for (i, (title, detail)) in [
            (
                "Primitives and strokes",
                "Filled polygons, outlines, and circular sectors",
            ),
            (
                "Color and transparency",
                "A four-corner gradient and overlapping alpha fills",
            ),
            (
                "Composed transforms",
                "Translation, rotation, and nonuniform scale",
            ),
            (
                "Texture sampling",
                "An image and the same 8 x 8 pattern with two filters",
            ),
            (
                "Text at different sizes",
                "Each size has its own raster at the current display scale",
            ),
            (
                "Pointer and background task",
                "Click below to stamp a circle; R clears the stamps",
            ),
        ]
        .iter()
        .enumerate()
        {
            gallery::panel(&mut scene, &library, &mut labels, i, title, detail);
        }
        scene.add(
            &shapes
                .triangle(
                    Vec2::new(-43.0, 35.0),
                    Vec2::new(0.0, -40.0),
                    Vec2::new(43.0, 35.0),
                )
                .fill_color(color::CYAN)
                .move_to(Vec2::new(120.0, 252.0)),
        );
        scene.add(
            &shapes
                .unit_hexagon()
                .fill_color(Vec4::new(1.0, 0.55, 0.16, 1.0))
                .scale(43.0)
                .move_to(Vec2::new(248.0, 252.0)),
        );
        scene.add(
            &shapes
                .unit_circle()
                .stroke_color(0.16, color::CYAN)
                .scale(43.0)
                .move_to(Vec2::new(376.0, 252.0)),
        );
        scene.add(
            &shapes
                .unit_circle()
                .sector(4.7)
                .stroke_color(0.5, color::YELLOW)
                .scale(43.0)
                .move_to(Vec2::new(500.0, 252.0)),
        );
        scene.add(
            &shapes
                .rectangle((Vec2::new(655.0, 200.0), Vec2::new(835.0, 310.0)))
                .fill_texture(&gradient),
        );
        for (offset, tint) in [
            (Vec2::new(-32.0, -16.0), Vec4::new(1.0, 0.15, 0.2, 0.65)),
            (Vec2::new(32.0, -16.0), Vec4::new(0.15, 1.0, 0.5, 0.65)),
            (Vec2::new(0.0, 30.0), Vec4::new(0.2, 0.45, 1.0, 0.65)),
        ] {
            scene.add(
                &shapes
                    .unit_circle()
                    .fill_color(tint)
                    .scale(45.0)
                    .move_to(Vec2::new(999.0, 247.0) + offset),
            );
        }
        for (i, stretch) in [Vec2::ONE, Vec2::new(1.5, 0.6), Vec2::new(0.7, 1.3)]
            .into_iter()
            .enumerate()
        {
            let center = Vec2::new(120.0 + i as f32 * 176.0, 513.0);
            scene.add(
                &shapes
                    .unit_circle()
                    .stroke_color(0.025, Vec4::new(0.3, 0.4, 0.5, 1.0))
                    .scale(62.0)
                    .move_to(center),
            );
            let parent = Affine2::from_scale_angle_translation(stretch * 40.0, angle, center);
            scene.add(&shapes.unit_quad().fill_texture(&gradient).transform(parent));
            scene.add(
                &shapes
                    .unit_circle()
                    .fill_color(color::WHITE)
                    .scale(0.15)
                    .move_to(Vec2::new(1.35, 0.0))
                    .transform(parent),
            );
        }
        for (i, (texture, caption)) in [
            (&photo, "Image"),
            (&nearest, "Nearest"),
            (&linear, "Linear"),
        ]
        .into_iter()
        .enumerate()
        {
            let origin = Vec2::new(657.0 + i as f32 * 173.0, 448.0);
            scene.add(
                &shapes
                    .rectangle((origin, origin + Vec2::new(130.0, 110.0)))
                    .fill_texture(texture),
            );
            scene.add(
                &labels
                    .text(caption, 16.0)
                    .move_to(origin + Vec2::new(0.0, 133.0)),
            );
        }
        for (size, y, text) in [
            (14.0, 711.0, "14 px  -  small labels"),
            (24.0, 748.0, "24 px  -  body text"),
            (40.0, 802.0, "40 px  -  headings"),
        ] {
            scene.add(&labels.text(text, size).move_to(Vec2::new(48.0, y)));
        }
        scene.add(
            &labels
                .text("right aligned", 16.0)
                .align(TextAlign::Right)
                .move_to(Vec2::new(550.0, 836.0)),
        );
        scene.add(
            &labels
                .text(
                    &format!(
                        "Animation: {}   |   Background ticks: {}",
                        if paused { "paused" } else { "running" },
                        ticks.get()
                    ),
                    17.0,
                )
                .move_to(Vec2::new(648.0, 710.0)),
        );
        scene.add(
            &shapes
                .rectangle((input_origin, input_end))
                .fill_color(Vec4::new(0.025, 0.035, 0.055, 1.0)),
        );
        for &p in &stamps {
            scene.add(
                &shapes
                    .unit_circle()
                    .fill_color(Vec4::new(0.15, 0.8, 1.0, 0.45))
                    .scale(10.0)
                    .move_to(p),
            );
        }
        if let Some(p) = pointer.filter(|p| inside(*p)) {
            scene.add(
                &shapes
                    .unit_circle()
                    .stroke_color(0.12, color::YELLOW)
                    .scale(14.0)
                    .move_to(p),
            );
        }
        scene.render();
        frame.present();
        frames += 1;
        if smoke && frames == 12 {
            break;
        }
    }
    background.handle().terminate();
    let _ = background.await;
    Ok(())
}
