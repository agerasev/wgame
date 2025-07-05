use std::{
    f32::consts::{FRAC_PI_3, PI},
    time::Instant,
};

use glam::{Mat4, Vec2};
use rgb::Rgb;
use wgame::{
    Runtime, WindowConfig,
    gfx::{self, ObjectExt, library::GeometryExt},
};
use wgame_utils::FrameCounter;

#[wgame::main]
async fn main(rt: Runtime) {
    env_logger::init();

    let task = rt
        .clone()
        .create_window(WindowConfig::default(), async move |mut window| {
            let gfx = gfx::Library::new(window.graphics())?;
            let [r, g, b] = [
                Rgb::new(1.0, 0.0, 0.0),
                Rgb::new(0.0, 1.0, 0.0),
                Rgb::new(0.0, 0.0, 1.0),
            ];
            let start_time = Instant::now();
            let mut fps = FrameCounter::default();
            while let Some(frame) = window.next_frame().await? {
                let angle = (2.0 * PI) * (Instant::now() - start_time).as_secs_f32() / 10.0;
                frame.render(
                    &gfx.triangle(
                        Vec2::new(0.0, 1.0),
                        Vec2::new((2.0 * FRAC_PI_3).sin(), (2.0 * FRAC_PI_3).cos()),
                        Vec2::new((4.0 * FRAC_PI_3).sin(), (4.0 * FRAC_PI_3).cos()),
                    )
                    .gradient([[b, r], [g, r + g - b]])
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
