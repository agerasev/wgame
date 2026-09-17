//! Borrowed 2D/3D views and a nested preview. Space pauses; Escape closes.
use wgame::{
    Library, Result, Window,
    canvas::{Event, Key},
    gfx::{Camera, DepthMode, Scene, Target, Viewport},
    glam::{Affine2, Affine3A, Vec2, Vec3, camera},
    prelude::*,
    rgb::Rgba,
};

fn draw_view(
    target: &mut impl Target,
    library: &Library,
    angle: f32,
    perspective: bool,
    pointer: Option<Vec2>,
) {
    let (width, height) = target.size();
    let camera = if perspective {
        Camera::new(
            target.state(),
            camera::rh::proj::directx::perspective(1.0, width as f32 / height as f32, 0.1, 100.0)
                * camera::rh::view::look_at_mat4(Vec3::new(2.0, -3.0, 2.5), Vec3::ZERO, Vec3::Z),
        )
    } else {
        target.camera()
    };
    let shapes = library.shapes();
    let mut scene = Scene::default();
    // Extending beyond the camera makes clipping visible at every panel edge.
    for i in -10..=10 {
        let x = i as f32 * 0.25;
        for (a, b) in [
            (Vec2::new(x, -3.0), Vec2::new(x, 3.0)),
            (Vec2::new(-3.0, x), Vec2::new(3.0, x)),
        ] {
            scene.add(
                &shapes
                    .line(a, b, 0.008)
                    .fill_color(Rgba::new(0.16, 0.22, 0.3, 1.0)),
            );
        }
    }
    let quad = shapes.unit_quad();
    scene.add(
        &quad
            .fill_color(Rgba::new(0.18, 0.65, 0.95, 1.0))
            .scale(0.7)
            .transform(Affine3A::from_rotation_z(angle))
            .move_to(Vec3::new(-0.3, 0.0, 0.15)),
    );
    scene.add(
        &quad
            .fill_color(Rgba::new(1.0, 0.55, 0.16, 1.0))
            .scale(0.5)
            .transform(Affine3A::from_rotation_z(-angle))
            .move_to(Vec3::new(0.35, 0.1, 0.4)),
    );

    // The caller chooses input routing; cameras receive local physical pixels.
    let point = pointer.and_then(|pixel| {
        if perspective {
            let (origin, direction) = camera.screen_ray(pixel, target.size())?;
            let distance = -origin.z / direction.z;
            (distance.is_finite() && distance >= 0.0)
                .then_some((origin + distance * direction).truncate())
        } else {
            camera.screen_to_world(pixel, target.size())
        }
    });
    if let Some(point) = point {
        scene.add(
            &shapes
                .unit_circle()
                .stroke_color(0.12, Rgba::new(1.0, 0.8, 0.3, 1.0))
                .scale(0.09)
                .move_to(point)
                .depth(DepthMode::Overlay),
        );
    }
    target.render(&camera, &scene.bake());
}

fn local_pointer<T: Target>(viewport: &Viewport<'_, T>, pointer: Option<Vec2>) -> Option<Vec2> {
    let local = viewport.to_local(pointer?);
    let (width, height) = viewport.size();
    (local.cmpge(Vec2::ZERO).all() && local.cmplt(Vec2::new(width as f32, height as f32)).all())
        .then_some(local)
}

#[wgame::window(title = "wgame: borrowed viewports", logical_size = (1000.0, 680.0))]
async fn main(mut window: Window<'_>) -> Result<()> {
    let library = Library::new(window.graphics());
    let font = library.make_font(&wgame::typography::FontData::new(
        include_bytes!("../../assets/free-sans-bold.ttf").to_vec(),
        0,
    )?);
    let mut scale = window.scale_factor() as f32;
    let mut raster = font.rasterize(18.0 * scale);
    let mut paused = false;
    let mut angle = 0.0;
    let mut last = wgame::app::time::Instant::now();
    #[cfg(not(target_arch = "wasm32"))]
    let smoke = std::env::args().any(|arg| arg == "--smoke");
    #[cfg(target_arch = "wasm32")]
    let smoke = false;
    let mut frames = 0;
    'frames: while let Some(mut frame) = window.next_frame().await? {
        for event in &frame.input().events {
            match event {
                Event::Key {
                    key: Key::Space,
                    pressed: true,
                    repeat: false,
                } => paused = !paused,
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
        if scale != frame.scale_factor() as f32 {
            scale = frame.scale_factor() as f32;
            raster = font.rasterize(18.0 * scale);
        }
        let pointer = frame
            .input()
            .pointer
            .filter(|_| frame.input().hovered)
            .map(|p| p * scale);
        let (width, height) = frame.size();
        let margin = (18.0 * scale).round() as u32;
        let top = (100.0 * scale).round() as u32;
        frame.clear(Rgba::new(0.025, 0.035, 0.055, 1.0));
        let mut labels = Vec::new();
        // Skip areas too small to fit nonempty viewports, including minimized layouts.
        if width > 3 * margin + 2 && height > top + 3 * margin + 2 {
            let side_by_side = width >= height;
            let panel_size = if side_by_side {
                ((width - 3 * margin) / 2, height - top - margin)
            } else {
                (width - 2 * margin, (height - top - 2 * margin) / 2)
            };
            for i in 0..2 {
                let origin = if side_by_side {
                    (margin + i * (panel_size.0 + margin), top)
                } else {
                    (margin, top + i * (panel_size.1 + margin))
                };
                let mut panel = frame.viewport(origin, panel_size)?;
                panel.clear(Rgba::new(0.055, 0.075, 0.11, 1.0));
                let local = local_pointer(&panel, pointer);
                draw_view(&mut panel, &library, angle, i == 1, local);
                if i == 0 && panel_size.0 > 4 * margin && panel_size.1 > 4 * margin {
                    let size = (panel_size.0 / 3, panel_size.1 / 3);
                    let offset = (
                        panel_size.0 - size.0 - margin,
                        panel_size.1 - size.1 - margin,
                    );
                    let mut nested = panel.viewport(offset, size)?;
                    nested.clear(Rgba::new(0.12, 0.08, 0.16, 1.0));
                    let local = local_pointer(&nested, pointer);
                    draw_view(&mut nested, &library, -angle, false, local);
                    labels.push((
                        "Nested view",
                        Vec2::new((origin.0 + offset.0) as f32, (origin.1 + offset.1) as f32),
                    ));
                }
                labels.push((
                    if i == 0 {
                        "2D camera"
                    } else {
                        "Perspective camera"
                    },
                    Vec2::new(origin.0 as f32, origin.1 as f32),
                ));
            }
        }
        // Parent rendering resumes at full size after the borrowed views end.
        frame.clear_depth();
        let camera = frame.physical_camera();
        let mut scene = frame.scene();
        scene.camera = camera;
        for (label, pos) in labels {
            scene.add(
                &raster
                    .text(label)
                    .scale(15.0 * scale)
                    .move_to(pos + Vec2::new(10.0, 24.0) * scale),
            );
        }
        for (label, y, size) in [
            ("Borrowed viewports", 34.0, 26.0),
            (
                if paused {
                    "Space: resume   |   Paused   |   Esc: close"
                } else {
                    "Space: pause   |   Move pointer to pick   |   Esc: close"
                },
                65.0,
                18.0,
            ),
            (
                "Shared shapes, separate cameras. Resize to rearrange the views.",
                87.0,
                14.0,
            ),
        ] {
            scene.add(
                &raster
                    .text(label)
                    .scale(size * scale)
                    .transform(Affine2::from_translation(Vec2::new(18.0, y) * scale)),
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
