use futures::join;
use wgame_app::{Runtime, WindowAttributes, app_main};

async fn main_(rt: Runtime) {
    log::info!("Started");

    async fn make_window_and_wait_closed(rt: &Runtime, index: usize) {
        rt.create_windowed_task(WindowAttributes::default(), async move |mut window| {
            log::info!("Window #{index} created");
            while window.request_redraw().await.is_some() {}
        })
        .await
        .unwrap()
        .await;
        log::info!("Window #{index} closed");
    }

    join!(
        make_window_and_wait_closed(&rt, 0),
        make_window_and_wait_closed(&rt, 1),
        make_window_and_wait_closed(&rt, 2),
    );

    log::info!("Closed");
}

app_main!(main_);
