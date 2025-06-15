use std::{
    cell::RefCell,
    mem,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

use crate::{
    app::{AppProxy, AppState},
    executor::{ExecutorProxy, TaskId, Timer},
};

/// Handle to underlying async runtime.
#[derive(Clone)]
pub struct Runtime {
    executor: Rc<RefCell<ExecutorProxy>>,
    state: Rc<RefCell<AppState>>,
}

impl Runtime {
    pub(crate) fn new(executor: Rc<RefCell<ExecutorProxy>>, app: AppProxy) -> Self {
        Self {
            executor,
            state: app.state,
        }
    }

    pub fn request_render(&self) -> RequestRenderFuture<'_> {
        if let Some(window) = self.state.borrow().window.as_ref() {
            window.request_redraw();
        }
        RequestRenderFuture { state: &self.state }
    }

    pub fn is_closed(&self) -> bool {
        self.state.borrow().close_requested
    }

    pub fn spawn<T: 'static, F: Future<Output = T> + 'static>(&self, future: F) -> JoinHandle<T> {
        let proxy = Rc::new(RefCell::new(JoinProxy::default()));
        let task_id = self.executor.borrow_mut().spawn({
            let proxy = proxy.clone();
            async move {
                let output = future.await;

                let mut proxy = proxy.borrow_mut();
                proxy.output = Some(output);
                if let Some(waker) = proxy.waker.take() {
                    waker.wake();
                }
            }
        });

        JoinHandle {
            _task_id: task_id,
            proxy,
        }
    }

    pub fn sleep(&self, timeout: Duration) -> SleepFuture {
        let timestamp = Instant::now().checked_add(timeout).unwrap();
        let timer = self.executor.borrow_mut().add_timer(timestamp);
        SleepFuture { timer }
    }
}

pub struct RequestRenderFuture<'a> {
    state: &'a RefCell<AppState>,
}

impl<'a> Future for RequestRenderFuture<'a> {
    type Output = Option<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.borrow_mut();
        if mem::replace(&mut state.redraw_requested, false) || state.close_requested {
            Poll::Ready(if !state.close_requested {
                Some(())
            } else {
                None
            })
        } else {
            state.redraw_waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

struct JoinProxy<T> {
    output: Option<T>,
    waker: Option<Waker>,
}

impl<T> Default for JoinProxy<T> {
    fn default() -> Self {
        Self {
            output: None,
            waker: None,
        }
    }
}

pub struct JoinHandle<T> {
    _task_id: TaskId,
    proxy: Rc<RefCell<JoinProxy<T>>>,
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut proxy = self.proxy.borrow_mut();
        if let Some(output) = proxy.output.take() {
            Poll::Ready(output)
        } else {
            proxy.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub struct SleepFuture {
    timer: Timer,
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.timer.timestamp {
            Poll::Ready(())
        } else {
            self.timer.waker.set(Some(cx.waker().clone()));
            Poll::Pending
        }
    }
}
