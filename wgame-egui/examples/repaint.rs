//! Event-driven canvas. Run with `--smoke` for bounded idle/deadline checks.
use std::time::Duration;
use wgame::{
    ContentFrame, WindowHost,
    app::{self, time::Instant},
    gfx::{Target, types::color},
};
use wgame_egui::{EguiWindow, egui};

#[wgame::window(title = "On-demand repaint", logical_size = (640.0, 480.0))]
async fn main(window: wgame::Window<'_>) -> wgame::Result<()> {
    let mut count = 0;
    let mut host = EguiWindow::new(window, move |ui, canvas| {
        egui::Panel::left("controls").show(ui, |ui| {
            if ui.button("Increment").clicked() {
                count += 1;
            }
            ui.label(format!("Count: {count}"));
        });
        egui::CentralPanel::default()
            .show(ui, |ui| canvas.show(ui))
            .inner
    });
    if std::env::args().any(|arg| arg == "--smoke") {
        smoke(&mut host).await?;
    } else {
        // Draw once, then sleep until an event or egui repaint deadline.
        loop {
            let Some(mut frame) = host.next_frame().await? else {
                break;
            };
            frame.clear(color::BLUE);
            frame.present();
            host.wait_for_update(None).await;
        }
    }
    Ok(())
}

async fn smoke<L: FnMut(&mut egui::Ui, &wgame_egui::Canvas) -> egui::Response>(
    host: &mut EguiWindow<'_, L>,
) -> wgame::Result<()> {
    use futures::future::{Either, select};
    // Let startup/hover repaints settle; idle must then stay asleep for 250 ms.
    let limit = Instant::now() + Duration::from_secs(5);
    let mut frames = 0;
    loop {
        assert!(
            Instant::now() < limit,
            "host never became idle ({frames} frames)"
        );
        let mut frame = host.next_frame().await?.expect("window remains open");
        frame.clear(color::BLUE);
        frame.present();
        frames += 1;
        if matches!(
            select(
                Box::pin(host.wait_for_update(None)),
                Box::pin(app::sleep(Duration::from_millis(250)))
            )
            .await,
            Either::Right(_)
        ) {
            break;
        }
    }
    eprintln!("Idle after {frames} frames; no redraw for 250 ms");
    // A delayed egui request wakes an already settled host at its deadline.
    // Egui subtracts its predicted frame time from the requested delay.
    let start = Instant::now();
    host.context()
        .request_repaint_after(Duration::from_millis(50));
    let wake = select(
        Box::pin(host.wait_for_update(None)),
        Box::pin(app::sleep(Duration::from_secs(1))),
    )
    .await;
    assert!(
        matches!(wake, Either::Left(_)),
        "egui timer failed to wake host"
    );
    drop(wake);
    assert!(
        start.elapsed() >= Duration::from_millis(20),
        "delayed repaint woke early"
    );
    eprintln!("Egui deadline woke after {:?}", start.elapsed());
    let mut frame = host.next_frame().await?.unwrap();
    frame.clear(color::BLUE);
    frame.present();
    // Explicit immediate requests are also supported, including after cancelling a wait.
    host.context().request_repaint();
    assert!(matches!(
        select(
            Box::pin(host.wait_for_update(None)),
            Box::pin(app::sleep(Duration::from_secs(1)))
        )
        .await,
        Either::Left(_)
    ));
    Ok(())
}
