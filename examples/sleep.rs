use std::time::Duration;

use wgame::{App, Runtime};

async fn main_(rt: Runtime) {
    println!("Going to sleep");
    rt.sleep(Duration::from_secs(1)).await;
    println!("Awakened");
}

fn main() {
    let app = App::new().unwrap();
    let rt = Runtime::new(app.proxy());
    app.proxy().spawn(main_(rt));
    app.run().unwrap();
}
