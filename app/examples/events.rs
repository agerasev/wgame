use futures::StreamExt;
use wgame_app::{Runtime, WindowAttributes, WindowEvent, run_main};

async fn main_(rt: Runtime) {
    env_logger::init();
    println!("Started");

    rt.create_window(WindowAttributes::default(), async |window| {
        println!("Window created");
        while let Some(event) = window.next().await {
            println!("Event: {:?}", event);
            if let WindowEvent::CloseRequested = event {
                break;
            }
        }
    })
    .await
    .unwrap();

    println!("Closed");
}

run_main!(main_);
