//! Raw event subscription without a graphics surface. `--redraws` includes OS
//! redraw notifications; `--smoke` exits after a short bounded observation.
use futures::StreamExt;
use wgame_app::{Window, input::Event, window_main};

async fn main_(window: Window<'_>) {
    log::info!("Window opened");

    let mut input = window.input();
    input.set_include_redraws(
        cfg!(not(target_arch = "wasm32")) && std::env::args().any(|arg| arg == "--redraws"),
    );
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::args().any(|arg| arg == "--smoke") {
        use futures::future::{Either, select};
        use std::time::Duration;
        let limit = wgame_app::time::Instant::now() + Duration::from_millis(250);
        loop {
            match select(
                Box::pin(input.next()),
                Box::pin(wgame_app::runtime::sleep_until(limit)),
            )
            .await
            {
                Either::Left((Some(event), _)) => log::info!("Event: {event:?}"),
                _ => return,
            }
            if wgame_app::time::Instant::now() >= limit {
                return;
            }
        }
    }
    while let Some(event) = input.next().await {
        log::info!("Event: {:?}", event);
        if event == Event::CloseRequested {
            break;
        }
    }

    log::info!("Window closed");
}

window_main!(main_);
