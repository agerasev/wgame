use std::time::Duration;

use wgame_app::{Runtime, run_main};

async fn main_(rt: Runtime) {
    env_logger::init();
    println!("Spawning new task");
    rt.spawn({
        let rt = rt.clone();
        async move {
            println!("Sleep task 1");
            rt.sleep(Duration::from_secs(1)).await;
            println!("Awakened task 1");
        }
    })
    .await;
    println!("Joined task 0");
}

run_main!(main_);
