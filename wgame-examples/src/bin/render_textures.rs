//! GPU drawing, snapshots, sampling, and composition without sampling an active target.
use wgame::{
    Library, Result, Window,
    canvas::{Event, Key},
    gfx::{
        Scene, Target,
        types::{Color, color},
    },
    glam::{Affine2, Vec2, Vec4},
    image::{Image, ImageWrite},
    prelude::*,
    texture::{RenderTexture, SampledTexture, Texture, TextureSettings},
};
use wgame_examples::{Labels, gallery};
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
    let (width, height) = target.size();
    let mut inset = target.viewport((width * 23 / 32, height / 16), (width / 5, height / 5))?;
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

fn sample(
    scene: &mut Scene,
    library: &Library,
    checker: &Texture,
    texture: &dyn SampledTexture,
    origin: Vec2,
    size: f32,
) {
    let quad = library
        .shapes()
        .rectangle((origin, origin + Vec2::splat(size)));
    scene.add(&quad.fill_texture(checker));
    scene.add(&quad.fill_texture(texture));
}

#[wgame::window(title = "wgame: render texture gallery", logical_size = (1200.0, 900.0))]
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
                        0.13
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
    let mut low = library
        .texturing()
        .render_texture((32, 32), TextureSettings::nearest())?;
    let mut composite = library
        .texturing()
        .render_texture((256, 256), TextureSettings::linear())?;
    let mut snapshots: Option<(Texture, Texture)> = None;
    let mut captures = 0;
    let mut paused = false;
    let mut angle = 0.0;
    let mut last = wgame::app::time::Instant::now();
    let smoke = cfg!(not(target_arch = "wasm32")) && std::env::args().any(|arg| arg == "--smoke");
    let mut frames = 0;
    'frames: while let Some(mut frame) = window.next_frame().await? {
        let mut capture = snapshots.is_none() || (smoke && frames == 6);
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
        draw_preview(&mut low, &library, &tile, angle)?;
        low.submit();
        if capture {
            // Readback submits pending drawing. CPU edits affect only the detached
            // snapshot, so the live target and the original upload keep their colors.
            let mut pixels = live.readback().await?;
            let original = library.make_texture(&pixels, TextureSettings::linear());
            for p in pixels.data_mut() {
                let grey = p.r.to_f32() * 0.2126 + p.g.to_f32() * 0.7152 + p.b.to_f32() * 0.0722;
                p.r = wgame::half::f16::from_f32(grey);
                p.g = p.r;
                p.b = p.r;
            }
            snapshots = Some((
                original,
                library.make_texture(&pixels, TextureSettings::linear()),
            ));
            captures += 1;
            log::info!("Render texture snapshot {captures} captured");
        } else {
            live.submit();
        }
        // A second allocation can sample the submitted live target. Never sample
        // the allocation currently bound as the render attachment.
        composite.clear(Vec4::ZERO);
        let camera = composite.camera();
        let mut composition = Scene::default();
        for i in 0..3 {
            let a = i as f32 * std::f32::consts::TAU / 3.0 + angle * 0.4;
            composition.add(
                &library
                    .shapes()
                    .unit_quad()
                    .fill_texture(&live)
                    .scale(0.5)
                    .transform(Affine2::from_angle(-a))
                    .move_to(Vec2::new(a.cos(), a.sin()) * 0.48),
            );
        }
        composite.render(&camera, &composition.bake());
        composite.submit();
        let (camera, scale) = gallery::camera(&mut frame);
        labels.set_scale(scale);
        frame.clear(gallery::BACKGROUND);
        let mut scene = frame.scene();
        scene.camera = camera;
        gallery::heading(
            &mut scene,
            &mut labels,
            "Render texture gallery",
            "S: capture snapshot    Space: pause / resume    Esc: close",
        );
        for (i, (title, detail)) in [
            (
                "Live render target",
                "Transparent drawing, plus a clipped inset viewport",
            ),
            (
                "Detached snapshot",
                "GPU readback uploaded into a separate CPU texture",
            ),
            (
                "Low-resolution sampling",
                "The same 32 x 32 target with nearest and linear filters",
            ),
            (
                "Texture coordinates",
                "Sample a center crop or flip the original horizontally",
            ),
            (
                "GPU composition",
                "One render target samples three copies of another",
            ),
            (
                "CPU pixel editing",
                "A grayscale copy of the snapshot; alpha is preserved",
            ),
        ]
        .iter()
        .enumerate()
        {
            gallery::panel(&mut scene, &library, &mut labels, i, title, detail);
        }
        sample(
            &mut scene,
            &library,
            &checker,
            &live,
            gallery::origin(0) + Vec2::new(20.0, 76.0),
            140.0,
        );
        scene.add(
            &labels
                .text("256 x 256 pixels", 20.0)
                .move_to(Vec2::new(224.0, 233.0)),
        );
        scene.add(
            &labels
                .text(
                    if paused {
                        "Paused"
                    } else {
                        "Updated every frame"
                    },
                    17.0,
                )
                .move_to(Vec2::new(224.0, 271.0)),
        );
        if let Some((original, grey)) = &snapshots {
            sample(
                &mut scene,
                &library,
                &checker,
                original,
                gallery::origin(1) + Vec2::new(20.0, 76.0),
                140.0,
            );
            scene.add(
                &labels
                    .text(&format!("Capture #{captures}"), 26.0)
                    .move_to(Vec2::new(824.0, 234.0)),
            );
            scene.add(
                &labels
                    .text("Press S to refresh", 17.0)
                    .move_to(Vec2::new(824.0, 272.0)),
            );
            sample(
                &mut scene,
                &library,
                &checker,
                grey,
                gallery::origin(5) + Vec2::new(20.0, 76.0),
                140.0,
            );
            sample(
                &mut scene,
                &library,
                &checker,
                &grey.multiply_color(Vec4::new(1.0, 0.6, 0.2, 0.8)),
                gallery::origin(5) + Vec2::new(208.0, 76.0),
                140.0,
            );
            scene.add(&labels.text("Gray", 16.0).move_to(Vec2::new(1028.0, 742.0)));
            scene.add(
                &labels
                    .text("+ tint", 16.0)
                    .move_to(Vec2::new(1028.0, 776.0)),
            );
        }
        for (i, (texture, title)) in [
            (low.sample(), "Nearest"),
            (
                low.sample().with_settings(TextureSettings::linear()),
                "Linear",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let origin = gallery::origin(2) + Vec2::new(20.0 + i as f32 * 262.0, 76.0);
            sample(&mut scene, &library, &checker, &texture, origin, 140.0);
            scene.add(
                &labels
                    .text(title, 16.0)
                    .move_to(origin + Vec2::new(149.0, 78.0)),
            );
        }
        for (i, (mapping, title)) in [
            (
                Affine2::from_scale_angle_translation(Vec2::splat(0.5), 0.0, Vec2::splat(0.25)),
                "Crop",
            ),
            (
                Affine2::from_scale_angle_translation(Vec2::new(-1.0, 1.0), 0.0, Vec2::X),
                "Flip X",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let origin = gallery::origin(3) + Vec2::new(20.0 + i as f32 * 262.0, 76.0);
            sample(
                &mut scene,
                &library,
                &checker,
                &live.sample().transform_coord(mapping),
                origin,
                140.0,
            );
            scene.add(
                &labels
                    .text(title, 16.0)
                    .move_to(origin + Vec2::new(149.0, 78.0)),
            );
        }
        sample(
            &mut scene,
            &library,
            &checker,
            &composite,
            gallery::origin(4) + Vec2::new(20.0, 76.0),
            140.0,
        );
        scene.add(
            &labels
                .text("Live -> composition", 20.0)
                .move_to(Vec2::new(224.0, 744.0)),
        );
        scene.add(
            &labels
                .text("No CPU readback in this pass", 17.0)
                .move_to(Vec2::new(224.0, 782.0)),
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
