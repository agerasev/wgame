#![no_std]

#[cfg(feature = "dump")]
extern crate std;

use core::{
    f32::consts::{FRAC_PI_3, PI, SQRT_2},
    time::Duration,
};
#[cfg(feature = "dump")]
use std::io::Write;

use glam::{Affine2, Vec2};
use rgb::Rgb;
#[cfg(feature = "dump")]
use wgame::image::ImageReadExt;
use wgame::{
    Library, Result, Window,
    app::time::Instant,
    font::Font,
    fs::read_bytes,
    gfx::{InstanceExt, types::color},
    image::Image,
    shapes::ShapeExt,
    utils::FrameCounter,
};

#[wgame::window(title = "Wgame example", size = (1200, 900), resizable = true, vsync = true)]
async fn main(mut window: Window<'_>) -> Result<()> {
    let gfx = Library::new(window.graphics())?;

    let texture = gfx
        .texture
        .texture(&Image::decode_auto(&read_bytes("assets/lenna.png").await?)?);
    let font = Font::new(read_bytes("assets/free-sans-bold.ttf").await?, 0)?;
    let mut font_raster = None;
    let mut text = None;
    let mut window_size = (0, 0);

    let triangle = gfx
        .shapes
        .triangle(
            Vec2::new(0.0, 1.0),
            Vec2::new((2.0 * FRAC_PI_3).sin(), (2.0 * FRAC_PI_3).cos()),
            Vec2::new((4.0 * FRAC_PI_3).sin(), (4.0 * FRAC_PI_3).cos()),
        )
        .texture(gfx.texture.gradient2([
            [color::BLUE, color::RED],
            [color::GREEN, color::RED + color::GREEN - color::BLUE],
        ]));

    let quad = gfx
        .shapes
        .quad(-Vec2::splat(0.5 * SQRT_2), Vec2::splat(0.5 * SQRT_2))
        .texture(texture.clone());

    let hexagon = gfx.shapes.hexagon(Vec2::ZERO, 1.0).color(color::BLUE);

    let grad = gfx
        .texture
        .gradient2([[color::WHITE, color::BLUE], [color::GREEN, color::RED]]);
    let circle = gfx.shapes.circle(Vec2::ZERO, 0.8).texture(grad.clone());
    let ring0 = gfx.shapes.ring(Vec2::ZERO, 0.8, 0.4).texture(grad.clone());
    let ring1 = gfx.shapes.ring(Vec2::ZERO, 0.8, 0.5).texture(grad.clone());

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
            let raster = font_raster.insert(gfx.text.texture(&font, height as f32 / 10.0));
            text = Some(raster.text("Hello, World!"));
        }

        frame.clear(Rgb::new(0.0, 0.0, 0.0));
        let angle = (2.0 * PI) * (Instant::now() - start_time).as_secs_f32() / 10.0;

        frame.push(triangle.transform(Affine2::from_scale_angle_translation(
            Vec2::splat(scale),
            angle,
            Vec2::new(-2.0 * scale, scale),
        )));
        frame.push(quad.transform(Affine2::from_scale_angle_translation(
            Vec2::splat(scale),
            angle,
            Vec2::new(0.0, scale),
        )));
        frame.push(hexagon.transform(Affine2::from_scale_angle_translation(
            Vec2::splat(scale),
            angle,
            Vec2::new(2.0 * scale, scale),
        )));
        frame.push(circle.transform(Affine2::from_scale_angle_translation(
            Vec2::splat(scale),
            10.0 * angle,
            Vec2::new(-2.0 * scale, -scale),
        )));
        frame.push(ring0.transform(Affine2::from_scale_angle_translation(
            Vec2::splat(scale),
            10.0 * angle,
            Vec2::new(0.0 * scale, -scale),
        )));
        frame.push(ring1.transform(Affine2::from_scale_angle_translation(
            Vec2::splat(scale),
            10.0 * angle,
            Vec2::new(2.0 * scale, -scale),
        )));
        if let Some(text) = &text {
            frame.push(text.transform(Affine2::from_scale_angle_translation(
                Vec2::splat(1.0 / window_size.1 as f32),
                0.0,
                Vec2::new(0.4, 0.3),
            )));
        }

        n_passes += frame.render()?;
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
