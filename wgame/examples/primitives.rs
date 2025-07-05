use std::{
    f32::consts::{FRAC_PI_3, PI},
    time::Instant,
};

use glam::{Affine3A, Quat, Vec2, Vec3};
use rgb::Rgb;
use wgame::{
    Runtime, Window, WindowConfig,
    gfx::{self, Library, Object, ObjectExt, library::GeometryExt},
};
use wgame_utils::FrameCounter;

#[wgame::main]
async fn main(rt: Runtime) {
    env_logger::init();

    let task = rt
        .clone()
        .create_window(WindowConfig::default(), async move |mut window: Window| {
            let gfx = gfx::Library::new(window.graphics())?;
            let start_time = Instant::now();
            let mut fps = FrameCounter::default();
            while let Some(frame) = window.next_frame().await? {
                frame.clear(Rgb::new(0.0, 0.0, 0.0));
                let angle = (2.0 * PI) * (Instant::now() - start_time).as_secs_f32() / 10.0;
                frame.render(
                    &triangle(&gfx).transform(Affine3A::from_scale_rotation_translation(
                        0.6 * (f32::sqrt(3.0) / 2.0) * Vec3::ONE,
                        Quat::from_rotation_z(-angle),
                        Vec3::new(-0.6, 0.6, 0.0),
                    )),
                );
                frame.render(
                    &quad(&gfx).transform(Affine3A::from_scale_rotation_translation(
                        0.6 * Vec3::ONE,
                        Quat::from_rotation_z(angle),
                        Vec3::new(0.6, 0.6, 0.0),
                    )),
                );
                frame.render(
                    &circle(&gfx).transform(Affine3A::from_scale_rotation_translation(
                        0.6 * Vec3::ONE,
                        Quat::from_rotation_z(10.0 * angle),
                        Vec3::new(-0.6, -0.6, 0.0),
                    )),
                );
                frame.render(
                    &ring(&gfx).transform(Affine3A::from_scale_rotation_translation(
                        0.6 * Vec3::ONE,
                        Quat::from_rotation_z(-10.0 * angle),
                        Vec3::new(0.6, -0.6, 0.0),
                    )),
                );
                fps.count();
            }
            Ok(())
        })
        .await
        .unwrap();

    task.await.unwrap();
}

fn triangle(gfx: &Library<'_>) -> impl Object {
    let [r, g, b] = [
        Rgb::new(1.0, 0.0, 0.0),
        Rgb::new(0.0, 1.0, 0.0),
        Rgb::new(0.0, 0.0, 1.0),
    ];
    gfx.triangle(
        Vec2::new(0.0, 1.0),
        Vec2::new((2.0 * FRAC_PI_3).sin(), (2.0 * FRAC_PI_3).cos()),
        Vec2::new((4.0 * FRAC_PI_3).sin(), (4.0 * FRAC_PI_3).cos()),
    )
    .gradient([[b, r], [g, r + g - b]])
}

fn quad(gfx: &Library<'_>) -> impl Object {
    gfx.quad(Vec2::new(-0.5, -0.5), Vec2::new(0.5, 0.5))
        .gradient([
            [Rgb::new(1.0, 1.0, 1.0), Rgb::new(0.0, 0.0, 1.0)],
            [Rgb::new(0.0, 1.0, 0.0), Rgb::new(1.0, 0.0, 0.0)],
        ])
}

fn circle(gfx: &Library<'_>) -> impl Object {
    gfx.circle(Vec2::ZERO, 0.5).gradient([
        [Rgb::new(1.0, 1.0, 1.0), Rgb::new(0.0, 0.0, 1.0)],
        [Rgb::new(0.0, 1.0, 0.0), Rgb::new(1.0, 0.0, 0.0)],
    ])
}

fn ring(gfx: &Library<'_>) -> impl Object {
    gfx.ring(Vec2::ZERO, 0.5, 0.25).gradient([
        [Rgb::new(1.0, 1.0, 1.0), Rgb::new(0.0, 0.0, 1.0)],
        [Rgb::new(0.0, 1.0, 0.0), Rgb::new(1.0, 0.0, 0.0)],
    ])
}
