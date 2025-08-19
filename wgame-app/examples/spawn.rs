use core::time::Duration;

use wgame_app::{app_main, sleep, spawn};

async fn main_() {
    println!("Spawning new task");
    spawn(async {
        println!("Sleep task 1");
        sleep(Duration::from_secs(2)).await;
        println!("Awakened task 1");
    });
    println!("Sleep task 0");
    sleep(Duration::from_secs(1)).await;
    println!("Awakened task 0");
}

app_main!(main_);
