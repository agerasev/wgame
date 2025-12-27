#![forbid(unsafe_code)]

#[cfg(feature = "dump")]
use std::io::Write;
use std::{
    f32::consts::{FRAC_PI_3, PI, SQRT_2},
    time::Duration,
};

#[cfg(feature = "dump")]
use wgame::image::ImageReadExt;
use wgame::{
    Library, Result, Window,
    app::time::Instant,
    gfx::types::color,
    glam::{Affine2, Vec2},
    prelude::*,
    shapes::ShapeExt,
    texture::TextureSettings,
    typography::TextAlign,
    utils::PeriodicTimer,
};

#[wgame::window(title = "Wgame example", size = (1200, 900), resizable = true, vsync = true)]
async fn main(mut window: Window<'_>) -> Result<()> {
    let gfx = Library::new(window.graphics());

    let texture = &gfx
        .load_texture("assets/lenna.png", TextureSettings::linear())
        .await?;

    let font = gfx.load_font("assets/free-sans-bold.ttf").await?;
    let mut font_raster = None;
    let mut text = None;
    let mut window_size = (0, 0);

    let triangle = &gfx
        .shapes()
        .triangle(
            Vec2::new(0.0, 1.0),
            Vec2::new((2.0 * FRAC_PI_3).sin(), (2.0 * FRAC_PI_3).cos()),
            Vec2::new((4.0 * FRAC_PI_3).sin(), (4.0 * FRAC_PI_3).cos()),
        )
        .with_texture(gfx.texturing().gradient2([
            [color::BLUE, color::RED],
            [color::GREEN, color::RED + color::GREEN - color::BLUE],
        ]));

    let quad = &gfx
        .shapes()
        .rectangle((-Vec2::splat(0.5 * SQRT_2), Vec2::splat(0.5 * SQRT_2)))
        .with_texture(texture);

    let hexagon = &gfx
        .shapes()
        .unit_hexagon()
        .transform(Affine2::from_scale_angle_translation(
            Vec2::ONE,
            0.0,
            Vec2::ZERO,
        ))
        .with_color(color::BLUE);

    let circle = &gfx
        .shapes()
        .unit_circle()
        .segment(2.0 * PI / 3.0)
        .with_texture(texture)
        .mul_color(color::YELLOW);
    let mut ring0 = gfx
        .shapes()
        .unit_ring(0.75)
        .with_texture(gfx.texturing().gradient2([[
            color::RED,
            color::MAGENTA,
            color::BLUE,
            color::RED,
        ]]));
    let ring1 = &gfx
        .shapes()
        .unit_ring(0.75)
        .with_texture(gfx.texturing().gradient2([
            [color::BLACK, color::BLACK, color::BLACK, color::BLACK],
            [color::RED, color::GREEN, color::BLUE, color::RED],
            [color::BLACK, color::BLACK, color::BLACK, color::BLACK],
        ]));

    #[cfg(feature = "dump")]
    std::fs::File::create("dump/atlas.png")?.write_all(
        &texture
            .atlas()
            .inner()
            .with_data(|img| img.slice((.., ..)).encode("png"))?,
    )?;

    let scale = 1.0 / 3.0;
    let start_time = Instant::now();
    let mut periodic = PeriodicTimer::new(Duration::from_secs(4));
    let mut n_frames = 0;
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

        frame.clear(color::BLACK);
        let mut scene = frame.scene();

        let angle = (2.0 * PI) * (Instant::now() - start_time).as_secs_f32() / 10.0;

        scene.add(&triangle.transform(Affine2::from_scale_angle_translation(
            Vec2::splat(scale),
            angle,
            Vec2::new(-2.0 * scale, scale),
        )));

        scene.add(&quad.transform(Affine2::from_scale_angle_translation(
            Vec2::splat(scale),
            angle,
            Vec2::new(0.0, scale),
        )));

        scene.add(&hexagon.transform(Affine2::from_scale_angle_translation(
            Vec2::splat(scale),
            angle,
            Vec2::new(2.0 * scale, scale),
        )));
        if let Some(text) = &text {
            scene.add(
                &text
                    .align(TextAlign::Center)
                    .transform(Affine2::from_scale_angle_translation(
                        Vec2::splat(text.metrics().size() * 1.0 / window_size.1 as f32),
                        0.0,
                        Vec2::new(2.0 * scale, scale),
                    ))
                    .order(1),
            );
        }

        scene.add(&circle.transform(Affine2::from_scale_angle_translation(
            Vec2::splat(0.8 * scale),
            -angle,
            Vec2::new(-2.0 * scale, -scale),
        )));

        let (seg_angle, rot_angle) = {
            let a = 5.0 * angle;
            let i = (a / (2.0 * PI)).floor() as u32;
            if i.is_multiple_of(2) {
                let r = a % (2.0 * PI);
                ((2.0 * PI) - r, -r)
            } else {
                (a % (2.0 * PI), 0.0)
            }
        };
        ring0.inner = ring0.inner.segment(seg_angle);
        scene.add(&ring0.transform(Affine2::from_scale_angle_translation(
            Vec2::splat(0.8 * scale),
            rot_angle - angle,
            Vec2::new(0.0 * scale, -scale),
        )));

        scene.add(&ring1.transform(Affine2::from_scale_angle_translation(
            Vec2::splat(0.8 * scale),
            -10.0 * angle,
            Vec2::new(2.0 * scale, -scale),
        )));

        {
            n_frames += 1;
            n_passes += scene.len();
            let dur = periodic.elapsed_periods();
            if !dur.is_zero() {
                log::info!(
                    "FPS: {},\tRPasses per frame: {}",
                    n_frames as f32 / dur.as_secs_f32(),
                    n_passes as f32 / n_frames as f32,
                );
                n_frames = 0;
                n_passes = 0;
            }
        }
    }
    Ok(())
}
