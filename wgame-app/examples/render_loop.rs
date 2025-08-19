use core::time::Duration;

use wgame_app::{Window, sleep, window_main};

async fn main_(mut window: Window<'_>) {
    log::info!("Window created");

    let mut counter = 0;
    while window.request_redraw().await.is_some() {
        log::info!("Frame #{counter}");
        counter += 1;
        sleep(Duration::from_millis(100)).await;
    }

    log::info!("Window closed");
}

window_main!(main_);
