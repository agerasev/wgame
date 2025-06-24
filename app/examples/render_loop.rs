use std::time::Duration;

use wgame_app::{Runtime, WindowAttributes, run_main};

async fn main_(rt: Runtime) {
    env_logger::init();
    println!("Started");

    rt.create_window(WindowAttributes::default(), {
        let rt = rt.clone();
        async move |window| {
            println!("Window created");
            let mut counter = 0;
            while let Some(()) = window.request_redraw().await {
                println!("Rendered frame #{counter}");
                counter += 1;
                for _ in window.events() {
                    // Skip all events
                }
                rt.sleep(Duration::from_millis(100)).await;
            }
        }
    })
    .await
    .unwrap();
    println!("Closed");
}

run_main!(main_);
