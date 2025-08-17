use wgame_app::{WindowAttributes, main, within_window};

async fn main_() {
    log::info!("Started");

    within_window(WindowAttributes::default(), async |mut window| {
        log::info!("Window created");
        while let Some(frame) = window.request_redraw().await {
            let _ = frame;
            // log::info!("Event: {:?}", event);
            todo!("Collect events in Frame")
        }
    })
    .await
    .unwrap();

    log::info!("Closed");
}

main!(main_);
