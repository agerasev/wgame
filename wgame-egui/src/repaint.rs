//! Coalesced egui repaint deadlines, also accepting cross-thread requests.
use futures::{future::poll_fn, task::AtomicWaker};
use std::{pin::Pin, sync::Mutex, task::Poll, time::Duration};
use wgame::app::{runtime::sleep_until, time::Instant};

#[derive(Default)]
pub(super) struct Repaint {
    deadline: Mutex<Option<Instant>>,
    waker: AtomicWaker,
}
impl Repaint {
    pub fn request(&self, delay: Duration) {
        let Some(deadline) = Instant::now().checked_add(delay) else {
            return;
        };
        {
            let mut pending = self.deadline.lock().unwrap();
            *pending = Some(pending.map_or(deadline, |old| old.min(deadline)));
        }
        // Runtime callbacks may reenter: never wake while holding the lock.
        self.waker.wake();
    }
    pub fn clear(&self) {
        self.deadline.lock().unwrap().take();
    }
    pub async fn wait(&self) {
        struct Registration<'a>(&'a AtomicWaker);
        impl Drop for Registration<'_> {
            fn drop(&mut self) {
                self.0.take();
            }
        }
        // A cancelled/completed wait must not leave the runtime task registered.
        let _registration = Registration(&self.waker);
        let mut timer = None;
        poll_fn(|cx| {
            self.waker.register(cx.waker());
            let deadline = *self.deadline.lock().unwrap();
            let Some(deadline) = deadline else {
                return Poll::Pending;
            };
            if deadline <= Instant::now() {
                return Poll::Ready(());
            }
            if timer.as_ref().is_none_or(|(at, _)| *at != deadline) {
                timer = Some((deadline, sleep_until(deadline)));
            }
            Pin::new(&mut timer.as_mut().unwrap().1).poll(cx)
        })
        .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::FutureExt;
    #[test]
    fn cancelling_a_wait_unregisters_its_waker() {
        use std::{
            sync::{
                Arc,
                atomic::{AtomicUsize, Ordering},
            },
            task::{Context, Wake, Waker},
        };
        #[derive(Default)]
        struct Counter(AtomicUsize);
        impl Wake for Counter {
            fn wake(self: Arc<Self>) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        let repaint = Repaint::default();
        let counter = Arc::new(Counter::default());
        let waker = Waker::from(counter.clone());
        let mut wait = Box::pin(repaint.wait());
        assert!(
            wait.as_mut()
                .poll(&mut Context::from_waker(&waker))
                .is_pending()
        );
        drop(wait);
        repaint.request(Duration::ZERO);
        assert_eq!(counter.0.load(Ordering::SeqCst), 0);
    }
    #[test]
    fn repaint_requests_coalesce_and_clear_without_polling_when_idle() {
        let repaint = Repaint::default();
        assert!(repaint.wait().now_or_never().is_none());
        repaint.request(Duration::from_secs(60));
        let later = *repaint.deadline.lock().unwrap();
        repaint.request(Duration::ZERO);
        assert!(*repaint.deadline.lock().unwrap() < later);
        assert!(repaint.wait().now_or_never().is_some());
        repaint.clear();
        assert!(repaint.wait().now_or_never().is_none());
    }
}
