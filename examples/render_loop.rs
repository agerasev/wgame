use wgame::{App, Runtime};

async fn main_(rt: Runtime) {
    println!("Started");
    while let Some(_render) = rt.request_render().await {
        println!("Rendered");
    }
    println!("Closed");
}

fn main() {
    let app = App::new().unwrap();
    let rt = Runtime::new(app.proxy());
    app.proxy().spawn(main_(rt));
    app.run().unwrap();
}
