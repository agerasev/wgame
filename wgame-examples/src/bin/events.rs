#![forbid(unsafe_code)]

use std::time::Duration;

use futures::{FutureExt, StreamExt, select_biased};
use glam::{Affine2, Vec2};
use rgb::Rgb;
use wgame::{
    Event, Library, Result, Window, gfx::types::color, prelude::*, shapes::ShapeExt, sleep,
    typography::TextAlign, utils::FrameCounter,
};

#[wgame::window(title = "Wgame example", size = (1200, 900), resizable = true, vsync = false)]
async fn main(mut window: Window<'_>) -> Result<()> {
    let gfx = Library::new(window.graphics());

    let font = gfx.load_font("assets/free-sans-bold.ttf").await?;
    let font_size = 32.0;
    let font_atlas = font.rasterize(font_size);
    let mut fps_text = "".to_string();
    let mut mouse_text = "Move your mouse in the window".to_string();

    let ring = &gfx
        .shapes()
        .unit_ring(0.5)
        .texture(gfx.texturing().gradient2([[
            color::RED,
            color::YELLOW,
            color::GREEN,
            color::CYAN,
            color::BLUE,
            color::MAGENTA,
            color::RED,
        ]]));

    let mut input = window.input();
    let mut mouse_pos = Vec2::ZERO;

    let update_duration = Duration::from_secs(1);
    let mut fps = FrameCounter::new(2 * update_duration);
    loop {
        let mut event = select_biased! {
            ev = input.next().fuse() => ev,
            () = sleep(update_duration).fuse() => None,
        };
        while let Some(ev) = event {
            match ev {
                Event::CursorMoved { position, .. } => {
                    mouse_pos = Vec2::new(position.x as f32, position.y as f32);
                    mouse_text = format!("Mouse pos: {},{}", position.x as i32, position.y as i32);
                }
                Event::CloseRequested => break,
                _ => (),
            }
            event = input.try_next();
        }

        let mut frame = match window.next_frame().await? {
            Some(frame) => frame,
            None => break,
        };
        let (width, height) = frame.size();

        while let Some(event) = input.try_next() {
            if let Event::CursorMoved { position, .. } = event {
                mouse_pos = Vec2::new(position.x as f32, position.y as f32);
                mouse_text = format!("Mouse pos: {},{}", position.x as i32, position.y as i32);
            }
        }

        frame.clear(Rgb::new(0.0, 0.0, 0.0));
        let mut renderer = frame.with_physical_camera();

        font_atlas
            .text(&mouse_text)
            .align(TextAlign::Center)
            .transform(Affine2::from_translation(Vec2::new(
                width as f32 / 2.0,
                height as f32 / 2.0,
            )))
            .draw(&mut renderer);

        ring.transform(
            Affine2::from_translation(mouse_pos) * Affine2::from_scale(Vec2::splat(32.0)),
        )
        .draw(&mut renderer);

        font_atlas
            .text(&fps_text)
            .align(TextAlign::Left)
            .transform(Affine2::from_translation(Vec2::new(0.0, font_size)))
            .draw(&mut renderer);

        if let Some(fps) = fps.count() {
            fps_text = format!("FPS: {fps}");
            println!("{}", fps_text);
        }
    }
    Ok(())
}
