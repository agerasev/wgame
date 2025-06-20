use futures::StreamExt;
use wgame::{Runtime, WindowAttributes};

#[wgame::main]
async fn main(rt: Runtime) {
    env_logger::init();
    println!("Started");
    let mut window = rt.create_window(WindowAttributes::default()).await.unwrap();
    println!("Window created");
    while let Some(event) = window.events().next().await {
        println!("Event: {:?}", event);
    }
    println!("Closed");
}
