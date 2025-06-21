use std::convert::Infallible;

use futures::join;
use wgame::{Runtime, WindowAttributes, surface::DummySurface};

#[wgame::main]
async fn main(rt: Runtime) {
    env_logger::init();
    println!("Started");

    async fn make_window_and_wait_closed(rt: &Runtime, index: usize) {
        let mut window = rt
            .create_window(WindowAttributes::default(), |_: &_| {
                Ok::<_, Infallible>(DummySurface)
            })
            .await
            .unwrap();
        println!("Window #{index} created");
        window.closed().await;
        println!("Window #{index} closed");
    }

    join!(
        make_window_and_wait_closed(&rt, 0),
        make_window_and_wait_closed(&rt, 1),
        make_window_and_wait_closed(&rt, 2),
    );

    println!("Closed");
}
