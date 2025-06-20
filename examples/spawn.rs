use std::time::Duration;

#[wgame::main]
async fn main(rt: wgame::Runtime) {
    env_logger::init();
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
