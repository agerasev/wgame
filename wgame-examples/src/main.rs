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
    gfx::{self, InstanceExt, color},
    img::image_to_texture,
    shapes::{Library, ShapeExt, gradient2},
    utils::FrameCounter,
};

#[wgame::main]
async fn main(rt: Runtime) {
    let task = rt
        .clone()
        .create_window(
            WindowConfig {
                gfx: gfx::Config {
                    present_mode: gfx::PresentMode::AutoNoVsync,
                },
                ..Default::default()
            },
            async move |mut window: Window| {
                let gfx = window.graphics().clone();
                let lib = Library::new(&gfx)?;
                let tex =
                    image_to_texture(&gfx, &read_bytes("assets/lenna.png").await.unwrap()).unwrap();

                let triangle = lib
                    .triangle(
                        Vec2::new(0.0, 1.0),
                        Vec2::new((2.0 * FRAC_PI_3).sin(), (2.0 * FRAC_PI_3).cos()),
                        Vec2::new((4.0 * FRAC_PI_3).sin(), (4.0 * FRAC_PI_3).cos()),
                    )
                    .texture(gradient2(
                        &gfx,
                        [
                            [color::BLUE, color::RED],
                            [color::GREEN, color::RED + color::GREEN - color::BLUE],
                        ],
                    ));

                let quad = lib
                    .quad(-Vec2::splat(0.5 * SQRT_2), Vec2::splat(0.5 * SQRT_2))
                    .texture(tex.clone());

                let hexagon = lib.hexagon(Vec2::ZERO, 1.0).color(color::BLUE);

                let grad = gradient2(
                    &gfx,
                    [[color::WHITE, color::BLUE], [color::GREEN, color::RED]],
                );
                let circle = lib.circle(Vec2::ZERO, 0.8).texture(grad.clone());
                let ring0 = lib.ring(Vec2::ZERO, 0.8, 0.4).texture(grad.clone());
                let ring1 = lib.ring(Vec2::ZERO, 0.8, 0.5).texture(grad.clone());

                let scale = 1.0 / 3.0;
                let start_time = Instant::now();
                let mut fps = FrameCounter::new(Duration::from_secs(4));
                while let Some(mut frame) = window.next_frame().await? {
                    let ctx = frame.context();

                    frame.clear(Rgb::new(0.0, 0.0, 0.0));
                    let angle = (2.0 * PI) * (Instant::now() - start_time).as_secs_f32() / 10.0;

                    frame.push(
                        &ctx,
                        triangle.transform(Affine2::from_scale_angle_translation(
                            Vec2::splat(scale),
                            angle,
                            Vec2::new(-2.0 * scale, scale),
                        )),
                    );
                    frame.push(
                        &ctx,
                        quad.transform(Affine2::from_scale_angle_translation(
                            Vec2::splat(scale),
                            angle,
                            Vec2::new(0.0, scale),
                        )),
                    );
                    frame.push(
                        &ctx,
                        hexagon.transform(Affine2::from_scale_angle_translation(
                            Vec2::splat(scale),
                            angle,
                            Vec2::new(2.0 * scale, scale),
                        )),
                    );
                    frame.push(
                        &ctx,
                        circle.transform(Affine2::from_scale_angle_translation(
                            Vec2::splat(scale),
                            10.0 * angle,
                            Vec2::new(-2.0 * scale, -scale),
                        )),
                    );
                    frame.push(
                        &ctx,
                        ring0.transform(Affine2::from_scale_angle_translation(
                            Vec2::splat(scale),
                            10.0 * angle,
                            Vec2::new(0.0 * scale, -scale),
                        )),
                    );
                    frame.push(
                        &ctx,
                        ring1.transform(Affine2::from_scale_angle_translation(
                            Vec2::splat(scale),
                            10.0 * angle,
                            Vec2::new(2.0 * scale, -scale),
                        )),
                    );

                    fps.count();
                }
                Ok(())
            },
        )
        .await
        .unwrap();

    task.await.unwrap();
}
