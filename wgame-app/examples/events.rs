use wgame_app::{Runtime, WindowAttributes, app_main};

async fn main_(rt: Runtime) {
    log::info!("Started");

    rt.create_windowed_task(WindowAttributes::default(), async |mut window| {
        log::info!("Window created");
        while let Some(frame) = window.request_redraw().await {
            let _ = frame;
            // log::info!("Event: {:?}", event);
            todo!("Collect events in Frame")
        }
    })
    .await
    .unwrap()
    .await;

    log::info!("Closed");
}

app_main!(main_);
