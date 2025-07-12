use core::time::Duration;

use wgame_app::{Runtime, main};

async fn main_(rt: Runtime) {
    log::info!("Going to sleep");
    rt.create_timer(Duration::from_secs(1)).await;
    log::info!("Awakened");
}

main!(main_);
