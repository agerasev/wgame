use futures::join;
use wgame_app::{WindowAttributes, app_main, within_window};

async fn main_() {
    log::info!("Started");

    async fn make_window_and_wait_closed(index: usize) {
        within_window(WindowAttributes::default(), async move |mut window| {
            log::info!("Window #{index} created");
            while window.request_redraw().await.is_some() {}
        })
        .await
        .unwrap();
        log::info!("Window #{index} closed");
    }

    join!(
        make_window_and_wait_closed(0),
        make_window_and_wait_closed(1),
        make_window_and_wait_closed(2),
    );

    log::info!("Closed");
}

app_main!(main_);
