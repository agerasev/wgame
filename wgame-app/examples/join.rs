use core::time::Duration;

use wgame_app::{main, sleep, spawn};

async fn main_() {
    log::info!("Spawning new task");
    spawn(async {
        log::info!("Sleep task 1");
        sleep(Duration::from_secs(1)).await;
        log::info!("Awakened task 1");
    })
    .await
    .unwrap();
    log::info!("Joined task 0");
}

main!(main_);
