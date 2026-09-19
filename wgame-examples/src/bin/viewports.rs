//! The same scene through orthographic, perspective, transformed, and nested views.
use wgame::{
    Library, Result, Window,
    canvas::{Event, Key},
    gfx::{Camera, DepthMode, Scene, Target, Viewport},
    glam::{Affine2, Affine3A, Vec2, Vec3, Vec4, camera},
    prelude::*,
};
use wgame_examples::{Labels, gallery};

#[derive(Clone, Copy)]
enum View {
    Overview,
    Perspective,
    Zoom,
}

fn draw_view(
    target: &mut impl Target,
    library: &Library,
    angle: f32,
    view: View,
    pointer: Option<Vec2>,
) {
    let (width, height) = target.size();
    let camera = match view {
        View::Perspective => Camera::new(
            target.state(),
            camera::rh::proj::directx::perspective(1.0, width as f32 / height as f32, 0.1, 100.0)
                * camera::rh::view::look_at_mat4(Vec3::new(2.0, -3.0, 2.5), Vec3::ZERO, Vec3::Z),
        ),
        View::Overview => target.camera(),
        View::Zoom => target
            .camera()
            .transform(Affine2::from_scale_angle_translation(
                Vec2::splat(2.2),
                0.35 * angle.sin(),
                Vec2::new(0.3, -0.2),
            )),
    };
    let shapes = library.shapes();
    let mut scene = Scene::default();
    // The grid and axes extend outside each view, making independent clipping visible.
    for i in -16..=16 {
        let x = i as f32 * 0.25;
        for (a, b) in [
            (Vec2::new(x, -4.0), Vec2::new(x, 4.0)),
            (Vec2::new(-4.0, x), Vec2::new(4.0, x)),
        ] {
            scene.add(
                &shapes
                    .line(a, b, 0.008)
                    .fill_color(Vec4::new(0.16, 0.22, 0.3, 1.0)),
            );
        }
    }
    scene.add(
        &shapes
            .line(Vec2::new(-4.0, 0.0), Vec2::new(4.0, 0.0), 0.025)
            .fill_color(Vec4::new(0.7, 0.24, 0.25, 1.0)),
    );
    scene.add(
        &shapes
            .line(Vec2::new(0.0, -4.0), Vec2::new(0.0, 4.0), 0.025)
            .fill_color(Vec4::new(0.18, 0.5, 0.75, 1.0)),
    );
    for i in 0..12 {
        let a = i as f32 * std::f32::consts::TAU / 12.0;
        let pos = Vec2::new(a.cos(), a.sin()) * 1.5;
        scene.add(
            &shapes
                .unit_hexagon()
                .fill_color(Vec4::new(0.2, 0.38 + i as f32 * 0.025, 0.45, 1.0))
                .scale(0.12)
                .move_to(pos),
        );
    }
    for (size, position, tint, rotation) in [
        (
            0.55,
            Vec3::new(-0.5, -0.15, 0.15),
            Vec4::new(0.18, 0.65, 0.95, 1.0),
            angle,
        ),
        (
            0.4,
            Vec3::new(0.4, 0.2, 0.4),
            Vec4::new(1.0, 0.55, 0.16, 1.0),
            -angle,
        ),
        (
            0.25,
            Vec3::new(0.0, -0.4, 0.65),
            Vec4::new(0.8, 0.3, 0.6, 1.0),
            angle * 0.6,
        ),
    ] {
        scene.add(
            &shapes
                .unit_quad()
                .fill_color(tint)
                .scale(size)
                .transform(Affine3A::from_rotation_z(rotation))
                .move_to(position),
        );
        // Upright strips connect the elevated quads to the ground plane.
        scene.add(
            &shapes
                .rectangle((Vec2::new(-0.0125, 0.0), Vec2::new(0.0125, position.z)))
                .fill_color(Vec4::new(0.6, 0.65, 0.7, 1.0))
                .transform(Affine3A::from_rotation_x(std::f32::consts::FRAC_PI_2))
                .move_to(Vec3::new(position.x, position.y, 0.0)),
        );
    }
    // Input is expressed in local physical pixels. Perspective picking intersects
    // a screen ray with the ground plane; transformed 2D views use the inverse camera.
    let point = pointer.and_then(|pixel| {
        if matches!(view, View::Perspective) {
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
                .stroke_color(0.15, Vec4::new(1.0, 0.85, 0.3, 1.0))
                .scale(0.09)
                .move_to(point)
                .depth(DepthMode::Overlay),
        );
        for direction in [Vec2::X, Vec2::Y] {
            scene.add(
                &shapes
                    .line(point - direction * 0.16, point + direction * 0.16, 0.015)
                    .fill_color(Vec4::new(1.0, 0.85, 0.3, 1.0))
                    .depth(DepthMode::Overlay),
            );
        }
    }
    target.render(&camera, &scene.bake());
}

fn local_pointer<T: Target>(viewport: &Viewport<'_, T>, pointer: Option<Vec2>) -> Option<Vec2> {
    let local = viewport.to_local(pointer?);
    let (width, height) = viewport.size();
    (local.cmpge(Vec2::ZERO).all() && local.cmplt(Vec2::new(width as f32, height as f32)).all())
        .then_some(local)
}

#[wgame::window(title = "wgame: viewport gallery", logical_size = (1200.0, 900.0))]
async fn main(mut window: Window<'_>) -> Result<()> {
    let library = Library::new(window.graphics());
    let mut labels = Labels::new(&library)?;
    let mut paused = false;
    let mut angle = 0.0;
    let mut last = wgame::app::time::Instant::now();
    let smoke = cfg!(not(target_arch = "wasm32")) && std::env::args().any(|arg| arg == "--smoke");
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
        let dpi = frame.scale_factor() as f32;
        let pointer = frame
            .input()
            .pointer
            .filter(|_| frame.input().hovered)
            .map(|p| p * dpi);
        let (camera, scale) = gallery::camera(&mut frame);
        labels.set_scale(scale);
        let size = Vec2::new(frame.size().0 as f32, frame.size().1 as f32);
        let offset = (size - gallery::SIZE * scale) * 0.5;
        frame.clear(gallery::BACKGROUND);
        let mut scene = Scene::default();
        gallery::heading(
            &mut scene,
            &mut labels,
            "Viewport gallery",
            "Space: pause / resume    Move pointer to pick the ground plane    Esc: close",
        );
        let panels = [
            (
                "Orthographic overview",
                "Grid, axes, orbit markers, and three elevated quads",
                View::Overview,
            ),
            (
                "Perspective projection",
                "The same scene with depth and ground-plane ray picking",
                View::Perspective,
            ),
            (
                "Zoom and camera rotation",
                "2.2x scale; the grid is clipped at this view's edges",
                View::Zoom,
            ),
            (
                "Nested viewport",
                "The inset clears and draws only its own rectangle",
                View::Overview,
            ),
        ];
        for (i, (title, detail, _)) in panels.iter().enumerate() {
            let origin = Vec2::new(
                30.0 + (i % 2) as f32 * 600.0,
                110.0 + (i / 2) as f32 * 380.0,
            );
            scene.add(
                &library
                    .shapes()
                    .rectangle((origin, origin + Vec2::new(540.0, 355.0)))
                    .fill_color(Vec4::new(0.06, 0.08, 0.11, 1.0)),
            );
            scene.add(
                &labels
                    .text(title, 22.0)
                    .move_to(origin + Vec2::new(18.0, 30.0)),
            );
            scene.add(
                &labels
                    .text(detail, 16.0)
                    .move_to(origin + Vec2::new(18.0, 56.0)),
            );
        }
        frame.render(&camera, &scene.bake());
        for (i, (_, _, view)) in panels.into_iter().enumerate() {
            let origin = Vec2::new(
                42.0 + (i % 2) as f32 * 600.0,
                188.0 + (i / 2) as f32 * 380.0,
            );
            let pixels = (offset + origin * scale).floor().as_uvec2();
            let extent = (Vec2::new(516.0, 263.0) * scale).floor().as_uvec2();
            if extent.min_element() == 0 {
                continue;
            }
            let mut panel = frame.viewport((pixels.x, pixels.y), (extent.x, extent.y))?;
            panel.clear(Vec4::new(0.035, 0.05, 0.075, 1.0));
            let nested_size = (extent.x * 2 / 5, extent.y / 2);
            let nested_origin = (extent.x - nested_size.0, extent.y - nested_size.1);
            let has_inset = i == 3 && nested_size.0 > 0 && nested_size.1 > 0;
            let local = local_pointer(&panel, pointer).filter(|p| {
                // Route a pointer inside the inset to that camera alone.
                !has_inset || p.x < nested_origin.0 as f32 || p.y < nested_origin.1 as f32
            });
            draw_view(&mut panel, &library, angle, view, local);
            if has_inset {
                let mut nested = panel.viewport(nested_origin, nested_size)?;
                nested.clear(Vec4::new(0.13, 0.07, 0.13, 1.0));
                let local = local_pointer(&nested, pointer);
                draw_view(&mut nested, &library, angle, View::Zoom, local);
            }
        }
        frame.present();
        frames += 1;
        if smoke && frames == 12 {
            break;
        }
    }
    Ok(())
}
