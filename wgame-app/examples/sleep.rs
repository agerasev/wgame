use std::time::Duration;

use wgame_app::{Runtime, run_main};

async fn main_(rt: Runtime) {
    env_logger::init();
    println!("Going to sleep");
    rt.create_timer(Duration::from_secs(1)).await;
    println!("Awakened");
}

run_main!(main_);
