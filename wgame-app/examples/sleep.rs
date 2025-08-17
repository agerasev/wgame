use core::time::Duration;

use wgame_app::{main, sleep};

async fn main_() {
    log::info!("Going to sleep");
    sleep(Duration::from_secs(1)).await;
    log::info!("Awakened");
}

main!(main_);
