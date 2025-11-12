use futures::StreamExt;
use wgame_app::{Window, input::Event, window_main};

async fn main_(window: Window<'_>) {
    log::info!("Window opened");

    let mut input = window.input();
    while let Some(event) = input.next().await {
        log::info!("Event: {:?}", event);
        if event == Event::CloseRequested {
            break;
        }
    }

    log::info!("Window closed");
}

window_main!(main_);
