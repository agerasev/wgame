use wgame_app::{Runtime, WindowAttributes, run_main};

async fn main_(rt: Runtime) {
    env_logger::init();
    println!("Started");

    rt.create_window(WindowAttributes::default(), async |mut window| {
        println!("Window created");
        while let Some(_frame) = window.next_frame().await {
            // println!("Event: {:?}", event);
            todo!("Collect events in Frame")
        }
    })
    .await
    .unwrap()
    .await;

    println!("Closed");
}

run_main!(main_);
