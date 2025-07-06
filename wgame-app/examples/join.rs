use std::time::Duration;

use wgame_app::{Runtime, main};

async fn main_(rt: Runtime) {
    env_logger::init();
    println!("Spawning new task");
    rt.create_task({
        let rt = rt.clone();
        async move {
            println!("Sleep task 1");
            rt.create_timer(Duration::from_secs(1)).await;
            println!("Awakened task 1");
        }
    })
    .await;
    println!("Joined task 0");
}

main!(main_);
