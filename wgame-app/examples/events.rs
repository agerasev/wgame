use futures::StreamExt;
use wgame_app::{Runtime, WindowAttributes, WindowEvent, run_main};

async fn main_(rt: Runtime) {
    env_logger::init();
    println!("Started");

    rt.create_window(WindowAttributes::default(), async |mut window| {
        println!("Window created");
        while let Some(event) = window.input.next().await {
            println!("Event: {:?}", event);
            if let WindowEvent::CloseRequested = event {
                break;
            }
        }
    })
    .await
    .unwrap()
    .await;

    println!("Closed");
}

run_main!(main_);
