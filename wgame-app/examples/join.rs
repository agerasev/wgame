use core::time::Duration;

use wgame_app::{Runtime, app_main};

async fn main_(rt: Runtime) {
    log::info!("Spawning new task");
    rt.create_task({
        let rt = rt.clone();
        async move {
            log::info!("Sleep task 1");
            rt.create_timer(Duration::from_secs(1)).await;
            log::info!("Awakened task 1");
        }
    })
    .await;
    log::info!("Joined task 0");
}

app_main!(main_);
