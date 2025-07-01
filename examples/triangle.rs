use std::{f32::consts::PI, time::Instant};

use glam::Mat4;
use wgame::{Runtime, app::WindowAttributes};
use wgame_gfx::{graphics::Graphics, object::ObjectExt, surface::Surface};
use wgame_utils::FrameCounter;

#[wgame::main]
async fn main(rt: Runtime) {
    env_logger::init();
    println!("Started");

    rt.clone()
        .create_window(WindowAttributes::default(), async move |window| {
            let mut surface = Surface::new(window).await.unwrap();
            let gfx = Graphics::from_surface(&surface).unwrap();

            let start_time = Instant::now();
            let mut fps = FrameCounter::default();
            while let Some(mut frame) = surface.next_frame().await.unwrap() {
                let angle = (2.0 * PI) * (Instant::now() - start_time).as_secs_f32() / 10.0;
                frame.render(&gfx.triangle().transform(Mat4::from_rotation_z(angle)));
                fps.count();
            }
        })
        .await
        .unwrap()
        .await;
    println!("Closed");
}
