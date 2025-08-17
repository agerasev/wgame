use core::time::Duration;

use wgame_app::{WindowAttributes, main, sleep, within_window};

async fn main_() {
    log::info!("Started");

    within_window(WindowAttributes::default(), {
        async |mut window| {
            log::info!("Window created");
            let mut counter = 0;
            while window.request_redraw().await.is_some() {
                log::info!("Frame #{counter}");
                counter += 1;
                sleep(Duration::from_millis(100)).await;
            }
        }
    })
    .await
    .unwrap();

    log::info!("Closed");
}

main!(main_);
