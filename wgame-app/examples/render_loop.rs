use core::time::Duration;

use wgame_app::{Runtime, WindowAttributes, main};

async fn main_(rt: Runtime) {
    log::info!("Started");

    rt.create_windowed_task(WindowAttributes::default(), {
        let rt = rt.clone();
        async move |mut window| {
            log::info!("Window created");
            let mut counter = 0;
            while window.request_redraw().await.is_some() {
                log::info!("Frame #{counter}");
                counter += 1;
                rt.create_timer(Duration::from_millis(100)).await;
            }
        }
    })
    .await
    .unwrap()
    .await;

    log::info!("Closed");
}

main!(main_);
