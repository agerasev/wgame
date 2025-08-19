use wgame_app::{Window, window_main};

async fn main_(mut window: Window<'_>) {
    log::info!("Window opened");

    while let Some(frame) = window.request_redraw().await {
        let _ = frame;
        // log::info!("Event: {:?}", event);
        todo!("Collect events in Frame")
    }

    log::info!("Window closed");
}

window_main!(main_);
