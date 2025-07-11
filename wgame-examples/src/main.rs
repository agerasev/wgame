#![no_std]

use core::{
    f32::consts::{FRAC_PI_3, PI, SQRT_2},
    time::Duration,
};

use glam::{Affine2, Vec2};
use rgb::Rgb;
use wgame::{
    Runtime, Window, WindowConfig,
    app::{deps::log, timer::Instant},
    fs::read_bytes,
    gfx::{Object, ObjectExt, Texture, types::color},
    img::image_to_texture,
    shapes::{Library, ShapeExt},
    utils::FrameCounter,
};

#[wgame::main]
async fn main(rt: Runtime) {
    let task = rt
        .clone()
        .create_window(WindowConfig::default(), async move |mut window: Window| {
            let gfx = Library::new(window.graphics())?;
            let tex = image_to_texture(
                window.graphics(),
                &read_bytes("assets/lenna.png").await.unwrap(),
            )
            .unwrap();
            let scale = 1.0 / 3.0;
            let start_time = Instant::now();
            let mut fps = FrameCounter::new(Duration::from_secs(4));
            while let Some(frame) = window.next_frame().await? {
                frame.clear(Rgb::new(0.0, 0.0, 0.0));
                let angle = (2.0 * PI) * (Instant::now() - start_time).as_secs_f32() / 10.0;
                frame.render(
                    &triangle(&gfx).transform(Affine2::from_scale_angle_translation(
                        Vec2::splat(scale),
                        -angle,
                        Vec2::new(-2.0 * scale, scale),
                    )),
                );
                frame.render(
                    &quad(&gfx, &tex).transform(Affine2::from_scale_angle_translation(
                        Vec2::splat(scale),
                        angle,
                        Vec2::new(0.0, scale),
                    )),
                );
                frame.render(
                    &hexagon(&gfx).transform(Affine2::from_scale_angle_translation(
                        Vec2::splat(scale),
                        angle,
                        Vec2::new(2.0 * scale, scale),
                    )),
                );
                frame.render(
                    &circle(&gfx).transform(Affine2::from_scale_angle_translation(
                        Vec2::splat(scale),
                        10.0 * angle,
                        Vec2::new(-1.5 * scale, -scale),
                    )),
                );
                frame.render(&ring(&gfx).transform(Affine2::from_scale_angle_translation(
                    Vec2::splat(scale),
                    -10.0 * angle,
                    Vec2::new(1.5 * scale, -scale),
                )));
                fps.count();
            }
            Ok(())
        })
        .await
        .unwrap();

    task.await.unwrap();
}

fn triangle(gfx: &Library<'_>) -> impl Object {
    gfx.triangle(
        Vec2::new(0.0, 1.0),
        Vec2::new((2.0 * FRAC_PI_3).sin(), (2.0 * FRAC_PI_3).cos()),
        Vec2::new((4.0 * FRAC_PI_3).sin(), (4.0 * FRAC_PI_3).cos()),
    )
    .gradient([
        [color::BLUE, color::RED],
        [color::GREEN, color::RED + color::GREEN - color::BLUE],
    ])
}

fn quad<'a, 'b: 'a>(gfx: &'b Library<'a>, tex: &Texture<'a>) -> impl Object + 'a {
    gfx.quad(-Vec2::splat(0.5 * SQRT_2), Vec2::splat(0.5 * SQRT_2))
        .texture(tex.clone())
}

fn hexagon(gfx: &Library<'_>) -> impl Object {
    gfx.hexagon(Vec2::ZERO, 1.0)
        .gradient([[color::BLUE, color::MAGENTA], [color::CYAN, color::WHITE]])
}

fn circle(gfx: &Library<'_>) -> impl Object {
    gfx.circle(Vec2::ZERO, 0.8)
        .gradient([[color::WHITE, color::BLUE], [color::GREEN, color::RED]])
}

fn ring(gfx: &Library<'_>) -> impl Object {
    gfx.ring(Vec2::ZERO, 0.8, 0.4)
        .gradient([[color::WHITE, color::BLUE], [color::GREEN, color::RED]])
}
