use wgame_app::{Runtime, WindowAttributes, main};

async fn main_(rt: Runtime) {
    env_logger::init();
    println!("Started");

    rt.create_windowed_task(WindowAttributes::default(), async |mut window| {
        println!("Window created");
        while let Some(_) = window.request_redraw().await {
            // println!("Event: {:?}", event);
            todo!("Collect events in Frame")
        }
    })
    .await
    .unwrap()
    .await;

    println!("Closed");
}

main!(main_);
