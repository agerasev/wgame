use std::time::Duration;

use wgame_app::{Runtime, run};

async fn main_(rt: Runtime) {
    env_logger::init();
    println!("Going to sleep");
    rt.sleep(Duration::from_secs(1)).await;
    println!("Awakened");
}

run!(main, main_);
