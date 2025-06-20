use std::time::Duration;

use wgame::{Runtime, WindowAttributes};

#[wgame::main]
async fn main(rt: Runtime) {
    env_logger::init();
    println!("Started");
    let mut window = rt.create_window(WindowAttributes::default()).await.unwrap();
    println!("Window created");
    let mut counter = 0;
    while let Some(_render) = window.request_render().await {
        rt.sleep(Duration::from_millis(100)).await;
        println!("Rendered frame #{counter}");
        counter += 1;
    }
    println!("Closed");
}
