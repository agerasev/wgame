use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

use wgame::{App, Runtime};

#[derive(Default)]
struct LoaderInfo {
    waker: Option<Waker>,
    complete: bool,
}

struct CustomSleep {
    info: Arc<Mutex<LoaderInfo>>,
}

impl CustomSleep {
    fn new(delay: Duration) -> Self {
        let info = Arc::new(Mutex::new(LoaderInfo::default()));
        thread::spawn({
            let info = info.clone();
            move || {
                thread::sleep(delay);
                let mut guard = info.lock().unwrap();
                guard.complete = true;
                if let Some(waker) = guard.waker.take() {
                    waker.wake();
                }
            }
        });
        Self { info }
    }
}

impl Future for CustomSleep {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut guard = self.info.lock().unwrap();
        if guard.complete {
            Poll::Ready(())
        } else {
            guard.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

async fn main_(_rt: Runtime) {
    println!("Going to sleep");
    CustomSleep::new(Duration::from_secs(1)).await;
    println!("Awakened");
}

fn main() {
    env_logger::init();
    let app = App::new().unwrap();
    let rt = Runtime::new(app.proxy());
    app.proxy().spawn(main_(rt));
    app.run().unwrap();
}
