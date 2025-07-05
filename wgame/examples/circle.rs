use std::{f32::consts::PI, time::Instant};

use glam::{Mat4, Vec2};
use rgb::Rgb;
use wgame::{
    Runtime, WindowConfig,
    gfx::{self, library::GeometryExt},
};
use wgame_gfx::ObjectExt;
use wgame_utils::FrameCounter;

#[wgame::main]
async fn main(rt: Runtime) {
    env_logger::init();

    let task = rt
        .clone()
        .create_window(WindowConfig::default(), async move |mut window| {
            let gfx = gfx::Library::new(window.graphics())?;
            let start_time = Instant::now();
            let mut fps = FrameCounter::default();
            while let Some(frame) = window.next_frame().await? {
                let angle = (2.0 * PI) * (Instant::now() - start_time).as_secs_f32() / 1.0;
                frame.render(
                    &gfx.ring(Vec2::ZERO, 0.5, 0.25)
                        .gradient([
                            [Rgb::new(1.0, 1.0, 1.0), Rgb::new(0.0, 0.0, 1.0)],
                            [Rgb::new(0.0, 1.0, 0.0), Rgb::new(1.0, 0.0, 0.0)],
                        ])
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
