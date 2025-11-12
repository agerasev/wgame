#![forbid(unsafe_code)]

use std::time::Duration;

use glam::{Affine2, Vec2};
use rgb::Rgb;
use wgame::{
    Event, Library, Result, Window,
    font::Font,
    fs::read_bytes,
    gfx::{InstanceExt, types::color},
    shapes::ShapeExt,
    utils::FrameCounter,
};

#[wgame::window(title = "Wgame example", size = (1200, 900), resizable = true, vsync = true)]
async fn main(mut window: Window<'_>) -> Result<()> {
    let gfx = Library::new(window.graphics())?;

    let font = Font::new(read_bytes("assets/free-sans-bold.ttf").await?, 0)?;
    let mut font_atlas = None;
    let mut text = "0,0".to_string();

    let ring = gfx.shapes.ring(Vec2::ZERO, 0.8, 0.4).texture(
        gfx.texture
            .gradient2([[color::WHITE, color::BLUE], [color::GREEN, color::RED]]),
    );

    let mut input = window.input();
    let mut mouse_pos = Vec2::ZERO;

    let mut fps = FrameCounter::new(Duration::from_secs(4));
    while let Some(mut frame) = window.next_frame().await? {
        if let Some((_width, height)) = frame.resized() {
            let _ = font_atlas.insert(gfx.text.texture(&font, height as f32 / 10.0));
        }
        let (width, height) = frame.size();

        while let Some(event) = input.try_next() {
            if let Event::CursorMoved { position, .. } = event {
                let pos = position.cast::<i32>();
                mouse_pos = Vec2::new(
                    2.0 * (position.x as f32 / width as f32) - 1.0,
                    1.0 - 2.0 * (position.y as f32 / height as f32),
                ) * frame.viewport_size();
                text = format!("{},{}", pos.x, pos.y);
            }
        }

        frame.clear(Rgb::new(0.0, 0.0, 0.0));

        if let Some(atlas) = &font_atlas {
            frame.push(
                atlas
                    .text(&text)
                    .transform(Affine2::from_scale_angle_translation(
                        Vec2::splat(1.0 / height as f32),
                        0.0,
                        Vec2::new(0.4, 0.3),
                    )),
            );
        }

        frame.push(&ring.transform(
            Affine2::from_translation(mouse_pos) * Affine2::from_scale(Vec2::splat(0.1)),
        ));

        if let Some(fps) = fps.count() {
            println!("FPS: {fps}");
        }
    }
    Ok(())
}
