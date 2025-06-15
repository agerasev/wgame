use std::time::Duration;

use wgame::{App, Runtime};

async fn main_(rt: Runtime) {
    println!("Spawning new task");
    rt.spawn({
        let rt = rt.clone();
        async move {
            println!("Sleep task 1");
            rt.sleep(Duration::from_secs(2)).await;
            println!("Awakened task 1");
        }
    });
    println!("Sleep task 0");
    rt.sleep(Duration::from_secs(1)).await;
    println!("Awakened task 0");
}

fn main() {
    let app = App::new().unwrap();
    let rt = Runtime::new(app.proxy());
    app.proxy().spawn(main_(rt));
    app.run().unwrap();
}
