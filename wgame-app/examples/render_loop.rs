use std::time::Duration;

use futures::StreamExt;
use wgame_app::{Runtime, WindowAttributes, WindowEvent, run_main};

async fn main_(rt: Runtime) {
    env_logger::init();
    println!("Started");

    rt.create_window(WindowAttributes::default(), {
        let rt = rt.clone();
        async move |mut window| {
            println!("Window created");
            let mut counter = 0;
            'render_loop: loop {
                while let Some(event) = window.input.next().await {
                    match event {
                        WindowEvent::RedrawRequested => {
                            println!("Rendered frame #{counter}");
                            counter += 1;
                            rt.sleep(Duration::from_millis(100)).await;
                            window.surface.request_redraw();
                        }
                        WindowEvent::CloseRequested => break 'render_loop,
                        _ => (),
                    }
                }
            }
        }
    })
    .await
    .unwrap()
    .await;

    println!("Closed");
}

run_main!(main_);
