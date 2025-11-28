#![forbid(unsafe_code)]

use std::time::Duration;

use glam::{Affine2, Vec2};
use rgb::Rgb;
use wgame::{
    Event, Library, Result, Window, gfx::types::color, prelude::*, shapes::ShapeExt,
    utils::FrameCounter,
};

#[wgame::window(title = "Wgame example", size = (1200, 900), resizable = true)]
async fn main(mut window: Window<'_>) -> Result<()> {
    let gfx = Library::new(window.graphics());

    let font = gfx.load_font("assets/free-sans-bold.ttf").await?;
    let font_size = 32.0;
    let font_atlas = font.rasterize(font_size);
    let mut text = "Move your mouse in the window".to_string();

    let ring = gfx.shapes().ring(Vec2::ZERO, 1.0, 0.5).texture(
        gfx.texturing()
            .gradient2([[color::WHITE, color::BLUE], [color::GREEN, color::RED]]),
    );

    let mut input = window.input();
    let mut mouse_pos = Vec2::ZERO;

    let mut fps = FrameCounter::new(Duration::from_secs(4));
    while let Some(mut frame) = window.next_frame().await? {
        let (width, height) = frame.size();

        while let Some(event) = input.try_next() {
            if let Event::CursorMoved { position, .. } = event {
                mouse_pos = Vec2::new(position.x as f32, position.y as f32);
                text = format!("Mouse pos: {},{}", position.x as i32, position.y as i32);
            }
        }

        frame.clear(Rgb::new(0.0, 0.0, 0.0));
        let mut render = frame.with_physical_camera();

        render.add(
            font_atlas
                .text(&text)
                .transform(Affine2::from_translation(Vec2::new(
                    width as f32 / 2.0,
                    height as f32 / 2.0,
                ))),
        );

        render.add(ring.transform(
            Affine2::from_translation(mouse_pos) * Affine2::from_scale(Vec2::splat(32.0)),
        ));

        if let Some(fps) = fps.count() {
            println!("FPS: {fps}");
        }
    }
    Ok(())
}
