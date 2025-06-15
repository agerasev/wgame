use std::time::Duration;

use wgame::{App, Runtime, WindowAttributes};

async fn main_(rt: Runtime) {
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

fn main() {
    let app = App::new().unwrap();
    let rt = Runtime::new(app.proxy());
    app.proxy().spawn(main_(rt));
    app.run().unwrap();
}
