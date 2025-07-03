use std::{f32::consts::PI, time::Instant};

use wgame::{Runtime, app::WindowAttributes};
use wgame_gfx::{graphics::Graphics, surface::Surface};
use wgame_utils::FrameCounter;

#[wgame::main]
async fn main(rt: Runtime) {
    env_logger::init();
    println!("Started");

    rt.clone()
        .create_window(WindowAttributes::default(), async move |mut window| {
            println!("Window created");

            let surface = Surface::new(&window).await.unwrap();
            println!("Surface created");

            let gfx = Graphics::new(&surface).unwrap();
            println!("Graphics created");

            // let start_time = Instant::now();
            let mut fps = FrameCounter::default();
            while let Some(frame) = window.next_frame().await {
                let mut frame = surface.create_frame(frame).unwrap();
                // let angle = (2.0 * PI) * (Instant::now() - start_time).as_secs_f32() / 10.0;
                frame.render(&gfx.triangle());
                fps.count();
            }
        })
        .await
        .unwrap()
        .await;
    println!("Closed");
}
