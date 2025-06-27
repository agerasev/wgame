use futures::join;
use wgame_app::{Runtime, WindowAttributes, run_main};

async fn main_(rt: Runtime) {
    env_logger::init();
    println!("Started");

    async fn make_window_and_wait_closed(rt: &Runtime, index: usize) {
        rt.create_window(WindowAttributes::default(), async move |mut window| {
            println!("Window #{index} created");
            while let Some(_frame) = window.next_frame(&mut ()).await.unwrap() {}
        })
        .await
        .unwrap()
        .await;
        println!("Window #{index} closed");
    }

    join!(
        make_window_and_wait_closed(&rt, 0),
        make_window_and_wait_closed(&rt, 1),
        make_window_and_wait_closed(&rt, 2),
    );

    println!("Closed");
}

run_main!(main_);
