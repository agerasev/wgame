use core::time::Duration;

use wgame_app::{Runtime, app_main};

async fn main_(rt: Runtime) {
    println!("Spawning new task");
    rt.create_task({
        let rt = rt.clone();
        async move {
            println!("Sleep task 1");
            rt.create_timer(Duration::from_secs(2)).await;
            println!("Awakened task 1");
        }
    });
    println!("Sleep task 0");
    rt.create_timer(Duration::from_secs(1)).await;
    println!("Awakened task 0");
}

app_main!(main_);
