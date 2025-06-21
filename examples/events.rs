use std::convert::Infallible;

use futures::StreamExt;
use wgame::{Runtime, WindowAttributes, surface::DummySurface};

#[wgame::main]
async fn main(rt: Runtime) {
    env_logger::init();
    println!("Started");
    let mut window = rt
        .create_window(WindowAttributes::default(), |_: &_| {
            Ok::<_, Infallible>(DummySurface)
        })
        .await
        .unwrap();
    println!("Window created");
    while let Some(event) = window.events().next().await {
        println!("Event: {:?}", event);
    }
    println!("Closed");
}
