use glam::Vec2;
use rgb::Rgb;
use wgame::{
    Runtime, WindowConfig,
    gfx::{self, library::GeometryExt},
};
use wgame_utils::FrameCounter;

#[wgame::main]
async fn main(rt: Runtime) {
    env_logger::init();

    let task = rt
        .clone()
        .create_window(WindowConfig::default(), async move |mut window| {
            let gfx = gfx::Library::new(window.graphics())?;
            let mut fps = FrameCounter::default();
            while let Some(frame) = window.next_frame().await? {
                frame.render(&gfx.circle(Vec2::ZERO, 0.5).gradient([
                    [Rgb::new(1.0, 1.0, 1.0), Rgb::new(0.0, 0.0, 1.0)],
                    [Rgb::new(0.0, 1.0, 0.0), Rgb::new(1.0, 0.0, 0.0)],
                ]));
                fps.count();
            }
            Ok(())
        })
        .await
        .unwrap();

    task.await.unwrap();
}
