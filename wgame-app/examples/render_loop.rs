use std::time::Duration;

use wgame_app::{Runtime, WindowAttributes, run_main};

async fn main_(rt: Runtime) {
    env_logger::init();
    println!("Started");

    rt.create_window(WindowAttributes::default(), {
        let rt = rt.clone();
        async move |mut window| {
            println!("Window created");
            let mut counter = 0;
            while let Some(_frame) = window.next_frame(&mut ()).await.unwrap() {
                println!("Rendered frame #{counter}");
                counter += 1;
                rt.sleep(Duration::from_millis(100)).await;
            }
        }
    })
    .await
    .unwrap()
    .await;

    println!("Closed");
}

run_main!(main_);
