use std::time::Duration;

#[wgame::main]
async fn main(rt: wgame::Runtime) {
    env_logger::init();
    println!("Going to sleep");
    rt.sleep(Duration::from_secs(1)).await;
    println!("Awakened");
}
