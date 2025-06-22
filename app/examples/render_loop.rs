use std::{convert::Infallible, time::Duration};

use wgame_app::{Runtime, WindowAttributes, run_main, surface::DummySurface};

async fn main_(rt: Runtime) {
    env_logger::init();
    println!("Started");
    let mut window = rt
        .create_window(WindowAttributes::default(), |_: &_| {
            Ok::<_, Infallible>(DummySurface)
        })
        .await
        .unwrap();
    println!("Window created");
    let mut counter = 0;
    while window
        .render(|_: &mut _| {
            println!("Rendered frame #{counter}");
            counter += 1;
            Ok::<_, Infallible>(())
        })
        .await
        .unwrap()
        .is_some()
    {
        rt.sleep(Duration::from_millis(100)).await;
    }
    println!("Closed");
}

run_main!(main_);
