use std::time::Duration;

use wgame_app::{Runtime, WindowAttributes, main};

async fn main_(rt: Runtime) {
    env_logger::init();
    println!("Started");

    rt.create_windowed_task(WindowAttributes::default(), {
        let rt = rt.clone();
        async move |mut window| {
            println!("Window created");
            let mut counter = 0;
            while let Some(_) = window.request_redraw().await {
                println!("Rendered frame #{counter}");
                counter += 1;
                rt.create_timer(Duration::from_millis(100)).await;
            }
        }
    })
    .await
    .unwrap()
    .await;

    println!("Closed");
}

main!(main_);
