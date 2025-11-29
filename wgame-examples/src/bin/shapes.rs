#![forbid(unsafe_code)]

#[cfg(feature = "dump")]
use std::io::Write;
use std::{
    f32::consts::{FRAC_PI_3, PI, SQRT_2},
    time::Duration,
};

use glam::{Affine2, Vec2};
use rgb::Rgb;
#[cfg(feature = "dump")]
use wgame::image::ImageReadExt;
use wgame::{
    Library, Result, Window,
    app::time::Instant,
    gfx::{Object, types::color},
    prelude::*,
    shapes::ShapeExt,
    texture::TextureSettings,
    typography::TextAlign,
    utils::FrameCounter,
};

#[wgame::window(title = "Wgame example", size = (1200, 900), resizable = true, vsync = true)]
async fn main(mut window: Window<'_>) -> Result<()> {
    let gfx = Library::new(window.graphics());

    let texture = gfx
        .load_texture("assets/lenna.png", TextureSettings::linear())
        .await?;
    let font = gfx.load_font("assets/free-sans-bold.ttf").await?;
    let mut font_raster = None;
    let mut text = None;
    let mut window_size = (0, 0);

    let triangle = gfx
        .shapes()
        .triangle(
            Vec2::new(0.0, 1.0),
            Vec2::new((2.0 * FRAC_PI_3).sin(), (2.0 * FRAC_PI_3).cos()),
            Vec2::new((4.0 * FRAC_PI_3).sin(), (4.0 * FRAC_PI_3).cos()),
        )
        .texture(gfx.texturing().gradient2([
            [color::BLUE, color::RED],
            [color::GREEN, color::RED + color::GREEN - color::BLUE],
        ]));

    let quad = gfx
        .shapes()
        .rectangle((-Vec2::splat(0.5 * SQRT_2), Vec2::splat(0.5 * SQRT_2)))
        .texture(texture.clone());

    let hexagon = gfx
        .shapes()
        .unit_hexagon()
        .transform(Affine2::from_scale_angle_translation(
            Vec2::ONE,
            0.0,
            Vec2::ZERO,
        ))
        .color(color::BLUE);

    let grad = gfx
        .texturing()
        .gradient2([[color::WHITE, color::BLUE], [color::GREEN, color::RED]]);
    let circle = gfx.shapes().circle(Vec2::ZERO, 0.8).texture(grad.clone());
    let ring0 = gfx
        .shapes()
        .ring(Vec2::ZERO, 0.8, 0.4)
        .texture(grad.clone());
    let ring1 = gfx
        .shapes()
        .ring(Vec2::ZERO, 0.8, 0.5)
        .texture(grad.clone());

    #[cfg(feature = "dump")]
    std::fs::File::create("dump/atlas.png")?.write_all(
        &texture
            .atlas()
            .inner()
            .with_data(|img| img.slice((.., ..)).encode("png"))?,
    )?;

    let scale = 1.0 / 3.0;
    let start_time = Instant::now();
    let mut fps = FrameCounter::new(Duration::from_secs(4));
    let mut n_passes = 0;
    while let Some(mut frame) = window.next_frame().await? {
        if let Some((width, height)) = frame.resized() {
            window_size = (width, height);
            let raster = font_raster.insert(font.rasterize(height as f32 / 10.0));
            text = Some(raster.text("Hello, World!"));

            #[cfg(feature = "dump")]
            std::fs::File::create("dump/font_atlas.png")?.write_all(
                &raster
                    .inner()
                    .atlas()
                    .inner()
                    .with_data(|img| img.slice((.., ..)).encode("png"))?,
            )?;
        }

        frame.clear(Rgb::new(0.0, 0.0, 0.0));
        let mut renderer = frame.with_unit_camera();

        let angle = (2.0 * PI) * (Instant::now() - start_time).as_secs_f32() / 10.0;

        triangle
            .transform(Affine2::from_scale_angle_translation(
                Vec2::splat(scale),
                angle,
                Vec2::new(-2.0 * scale, scale),
            ))
            .draw(&mut renderer);
        quad.transform(Affine2::from_scale_angle_translation(
            Vec2::splat(scale),
            angle,
            Vec2::new(0.0, scale),
        ))
        .draw(&mut renderer);
        hexagon
            .transform(Affine2::from_scale_angle_translation(
                Vec2::splat(scale),
                angle,
                Vec2::new(2.0 * scale, scale),
            ))
            .draw(&mut renderer);
        circle
            .transform(Affine2::from_scale_angle_translation(
                Vec2::splat(scale),
                10.0 * angle,
                Vec2::new(-2.0 * scale, -scale),
            ))
            .draw(&mut renderer);
        ring0
            .transform(Affine2::from_scale_angle_translation(
                Vec2::splat(scale),
                10.0 * angle,
                Vec2::new(0.0 * scale, -scale),
            ))
            .draw(&mut renderer);
        ring1
            .transform(Affine2::from_scale_angle_translation(
                Vec2::splat(scale),
                10.0 * angle,
                Vec2::new(2.0 * scale, -scale),
            ))
            .draw(&mut renderer);
        if let Some(text) = &text {
            text.align(TextAlign::Center)
                .transform(Affine2::from_scale_angle_translation(
                    Vec2::splat(1.0 / window_size.1 as f32),
                    0.0,
                    Vec2::new(2.0 * scale, scale),
                ))
                .draw(&mut renderer);
        }

        n_passes += frame.render().n_passes;
        if let Some(frames) = fps.count_ext() {
            log::info!(
                "FPS: {},\tRPasses per frame: {}",
                frames.per_second(),
                n_passes as f32 / frames.count as f32,
            );
            n_passes = 0;
        }
    }
    Ok(())
}
