#![forbid(unsafe_code)]

use std::time::Duration;

use futures::{FutureExt, StreamExt, select_biased};
use wgame::{
    Event, Library, Result, Window,
    gfx::types::color,
    glam::{Affine2, Vec2},
    prelude::*,
    typography::TextAlign,
    utils::PeriodicTimer,
};

#[wgame::window(title = "Wgame example", size = (1200, 900), resizable = true, vsync = false)]
async fn main(mut window: Window<'_>) -> Result<()> {
    let gfx = Library::new(window.graphics());

    let font = gfx.load_font("assets/free-sans-bold.ttf").await?;
    let font_size = 32.0;
    let font_atlas = font.rasterize(font_size);
    let mut fps_text = "".to_string();
    let mut mouse_text = "Move your mouse in the window".to_string();

    let ring = &gfx.shapes().unit_circle().stroke_texture(
        0.25,
        &gfx.texturing().gradient([
            color::RED,
            color::YELLOW,
            color::GREEN,
            color::CYAN,
            color::BLUE,
            color::MAGENTA,
            color::RED,
        ]),
    );

    let mut input = window.input();
    let mut mouse_pos = Vec2::ZERO;

    let mut periodic = PeriodicTimer::new(Duration::from_secs(1));
    let mut n_frames: u32 = 0;
    let mut need_redraw = true;
    loop {
        if !need_redraw {
            let mut event = select_biased! {
                event = input.next().fuse() => event,
                dur = periodic.wait_next().fuse() => {
                    let fps = n_frames as f32 / dur.as_secs_f32();
                    n_frames = 0;
                    fps_text = format!("FPS: {fps}");
                    println!("{}", fps_text);
                    need_redraw = true;
                    None
                },
            };
            while let Some(ev) = event {
                match ev {
                    Event::CursorMoved { position, .. } => {
                        mouse_pos = Vec2::new(position.x as f32, position.y as f32);
                        mouse_text =
                            format!("Mouse pos: {},{}", position.x as i32, position.y as i32);
                        need_redraw = true;
                    }
                    Event::CloseRequested => break,
                    _ => (),
                }
                event = input.try_next();
            }
        }
        if !need_redraw {
            continue;
        }
        need_redraw = false;

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

        frame.clear(color::BLACK);

        let camera = frame.physical_camera();
        let mut scene = frame.scene();
        scene.camera = camera;

        scene.add(
            &font_atlas
                .text(&mouse_text)
                .align(TextAlign::Center)
                .transform(Affine2::from_scale_angle_translation(
                    Vec2::splat(font_size),
                    0.0,
                    Vec2::new(width as f32 / 2.0, height as f32 / 2.0),
                )),
        );

        scene.add(
            &ring
                .transform(
                    Affine2::from_translation(Vec2::new(mouse_pos.x, mouse_pos.y))
                        * Affine2::from_scale(Vec2::splat(32.0)),
                )
                .order(-1),
        );

        scene.add(
            &font_atlas.text(&fps_text).align(TextAlign::Left).transform(
                Affine2::from_scale_angle_translation(
                    Vec2::splat(font_size),
                    0.0,
                    Vec2::new(0.0, font_size),
                ),
            ),
        );

        n_frames += 1;
    }
    Ok(())
}
