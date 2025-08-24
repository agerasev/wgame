use core::time::Duration;

use wgame_app::{app_main, sleep};

async fn main_() {
    log::info!("Going to sleep");
    sleep(Duration::from_secs(1)).await;
    log::info!("Awakened");
}

app_main!(main_);
