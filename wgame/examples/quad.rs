use std::{f32::consts::PI, time::Instant};

use glam::{Mat4, Vec2};
use rgb::Rgba;
use wgame::{
    Runtime, WindowConfig,
    gfx::{self, ObjectExt},
};
use wgame_gfx::library::GeometryExt;
use wgame_utils::FrameCounter;

#[wgame::main]
async fn main(rt: Runtime) {
    env_logger::init();

    let task = rt
        .clone()
        .create_window(WindowConfig::default(), async move |mut window| {
            let gfx = gfx::Library::new(window.graphics())?;

            let rect = [Vec2::new(-0.5, -0.5), Vec2::new(0.5, 0.5)];
            let colors = [
                [Rgba::new(1.0, 1.0, 1.0, 1.0), Rgba::new(0.0, 0.0, 1.0, 1.0)],
                [Rgba::new(0.0, 1.0, 0.0, 1.0), Rgba::new(1.0, 0.0, 0.0, 1.0)],
            ];

            let start_time = Instant::now();
            let mut fps = FrameCounter::default();
            while let Some(frame) = window.next_frame().await? {
                let angle = (2.0 * PI) * (Instant::now() - start_time).as_secs_f32() / 10.0;
                frame.render(
                    &gfx.quad()
                        .transform_to_rect(rect[0], rect[1])
                        .gradient(colors)
                        .transform(Mat4::from_rotation_z(angle)),
                );
                fps.count();
            }
            Ok(())
        })
        .await
        .unwrap();

    task.await.unwrap();
}
