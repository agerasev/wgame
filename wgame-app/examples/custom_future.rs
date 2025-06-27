use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

use wgame_app::{Runtime, run};

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
    env_logger::init();
    println!("Going to sleep");
    CustomSleep::new(Duration::from_secs(1)).await;
    println!("Awakened");
}

run!(main, main_);
