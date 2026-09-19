//! Render to a GPU texture, sample it, and capture a detached CPU snapshot.
use wgame::{
    Library, Result, Window,
    canvas::{Event, Key},
    gfx::{
        Scene, Target,
        types::{Color, color},
    },
    glam::{Affine2, Vec2, Vec4},
    image::Image,
    prelude::*,
    texture::{RenderTexture, SampledTexture, Texture, TextureSettings},
};
use wgame_examples::Labels;

fn draw_preview(
    target: &mut RenderTexture,
    library: &Library,
    tile: &Texture,
    angle: f32,
) -> Result<()> {
    target.clear(Vec4::ZERO);
    let camera = target.camera();
    let shapes = library.shapes();
    let mut scene = Scene::default();
    scene.add(
        &shapes
            .unit_circle()
            .fill_color(Vec4::new(0.1, 0.75, 1.0, 0.45))
            .scale(0.9),
    );
    scene.add(
        &shapes
            .unit_quad()
            .fill_texture(tile)
            .scale(0.48)
            .transform(Affine2::from_angle(angle)),
    );
    scene.add(
        &shapes
            .unit_circle()
            .stroke_color(0.15, color::WHITE)
            .scale(0.23)
            .move_to(Vec2::new(angle.cos(), angle.sin()) * 0.7),
    );
    target.render(&camera, &scene.bake());
    // A borrowed viewport works exactly as on a window, including clipped clears.
    let mut inset = target.viewport((184, 12), (60, 60))?;
    inset.clear(Vec4::new(1.0, 0.55, 0.12, 0.7));
    let camera = inset.camera();
    let mut scene = Scene::default();
    scene.add(
        &shapes
            .unit_quad()
            .fill_texture(tile)
            .scale(0.8)
            .transform(Affine2::from_angle(-angle)),
    );
    inset.render(&camera, &scene.bake());
    Ok(())
}

#[wgame::window(title = "wgame: render textures", logical_size = (1000.0, 650.0))]
async fn main(mut window: Window<'_>) -> Result<()> {
    let library = Library::new(window.graphics());
    let mut labels = Labels::new(&library)?;
    let tile = library.make_texture(
        &Image::with_data(
            (2, 2),
            vec![
                color::RED.to_rgba_f16(),
                color::YELLOW.to_rgba_f16(),
                color::BLUE.to_rgba_f16(),
                color::GREEN.to_rgba_f16(),
            ],
        ),
        TextureSettings::nearest(),
    );
    let checker = library.make_texture(
        &Image::with_data(
            (16, 16),
            (0..256)
                .map(|i| {
                    let c = if (i % 16 + i / 16) % 2 == 0 {
                        0.065
                    } else {
                        0.1
                    };
                    Vec4::new(c, c, c, 1.0).to_rgba_f16()
                })
                .collect::<Vec<_>>(),
        ),
        TextureSettings::nearest(),
    );
    let mut live = library
        .texturing()
        .render_texture((256, 256), TextureSettings::linear())?;
    let mut snapshot: Option<Texture> = None;
    let mut snapshots = 0;
    let mut paused = false;
    let mut angle = 0.0;
    let mut last = wgame::app::time::Instant::now();
    #[cfg(not(target_arch = "wasm32"))]
    let smoke = std::env::args().any(|arg| arg == "--smoke");
    #[cfg(target_arch = "wasm32")]
    let smoke = false;
    let mut frames = 0;
    'frames: while let Some(mut frame) = window.next_frame().await? {
        let mut capture = snapshot.is_none() || (smoke && frames == 6);
        for event in &frame.input().events {
            match event {
                Event::Key {
                    key: Key::Space,
                    pressed: true,
                    repeat: false,
                } => paused = !paused,
                Event::Key {
                    key: Key::Character('s'),
                    pressed: true,
                    repeat: false,
                } => capture = true,
                Event::Key {
                    key: Key::Escape,
                    pressed: true,
                    ..
                } => {
                    frame.discard();
                    break 'frames;
                }
                _ => (),
            }
        }
        let now = wgame::app::time::Instant::now();
        if !paused {
            angle += (now - last).as_secs_f32().min(0.1);
        }
        last = now;
        draw_preview(&mut live, &library, &tile, angle)?;
        if capture {
            // Readback submits the pending drawing and asynchronously downloads it.
            // Editing or dropping these CPU pixels cannot change the live target.
            let pixels = live.readback().await?;
            snapshot = Some(library.make_texture(&pixels, TextureSettings::linear()));
            snapshots += 1;
            log::info!("Render texture snapshot {snapshots} captured");
        } else {
            // Queue order makes this drawing visible when the window samples it.
            live.submit();
        }
        let scale = frame.scale_factor() as f32;
        labels.set_scale(scale);
        let (width, height) = frame.logical_size();
        let (width, height) = (width as f32, height as f32);
        frame.clear(Vec4::new(0.025, 0.035, 0.055, 1.0));
        let camera = frame.logical_camera();
        let mut scene = frame.scene();
        scene.camera = camera;
        let horizontal = width >= height;
        let available = Vec2::new((width - 48.0).max(1.0), (height - 155.0).max(1.0));
        let size = if horizontal {
            ((available.x - 24.0) / 2.0).min(available.y)
        } else {
            available.x.min((available.y - 52.0) / 2.0)
        }
        .max(1.0);
        let first = Vec2::new(24.0, 133.0);
        let second = first
            + if horizontal {
                Vec2::new(size + 24.0, 0.0)
            } else {
                Vec2::new(0.0, size + 52.0)
            };
        if let Some(snapshot) = &snapshot {
            for (origin, texture, label) in [
                (
                    first,
                    &live as &dyn SampledTexture,
                    "Live GPU texture".to_owned(),
                ),
                (
                    second,
                    snapshot as &dyn SampledTexture,
                    format!("CPU snapshot #{snapshots}"),
                ),
            ] {
                let quad = library
                    .shapes()
                    .rectangle((origin, origin + Vec2::splat(size)));
                scene.add(&quad.fill_texture(&checker));
                scene.add(&quad.fill_texture(texture));
                scene.add(
                    &labels
                        .text(&label, 18.0)
                        .move_to(origin - Vec2::new(0.0, 12.0)),
                );
            }
        }
        for (text, y, size) in [
            ("Render textures", 38.0, 28.0),
            (
                if paused {
                    "S: capture snapshot   |   Space: resume   |   Esc: close"
                } else {
                    "S: capture snapshot   |   Space: pause   |   Esc: close"
                },
                68.0,
                17.0,
            ),
            (
                "The snapshot stays frozen while the GPU texture keeps changing.",
                91.0,
                14.0,
            ),
        ] {
            scene.add(&labels.text(text, size).move_to(Vec2::new(24.0, y)));
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
