use alloc::rc::Rc;
use core::{
    cell::RefCell,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::FutureExt;
use winit::{error::OsError, window::WindowAttributes};

use crate::{
    Window,
    executor::{ExecutorProxy, TaskId},
    proxy::{AppProxy, CallbackTrigger, SharedCallState},
    timer::{Instant, Timer},
    window::create_window,
};

/// Handle to underlying async runtime.
#[derive(Clone)]
pub struct Runtime {
    app: AppProxy,
}

impl Runtime {
    pub fn new(app: AppProxy) -> Self {
        Self { app }
    }

    pub fn create_task<T: 'static, F: Future<Output = T> + 'static>(
        &self,
        future: F,
    ) -> JoinHandle<T> {
        let (task_id, proxy) = self.app.create_task(future);
        JoinHandle {
            task: task_id,
            executor: self.app.executor.clone(),
            proxy,
        }
    }

    pub fn create_timer(&self, timeout: Duration) -> Timer {
        let timestamp = Instant::now().checked_add(timeout).unwrap();
        self.app.timers.borrow_mut().add(timestamp)
    }

    pub async fn create_windowed_task<T: 'static, F: AsyncFnOnce(Window<'_>) -> T + 'static>(
        &self,
        attributes: WindowAttributes,
        window_main: F,
    ) -> Result<JoinHandle<T>, OsError> {
        let app = self.app.clone();
        let this = self.clone();
        let (task, proxy) = self
            .app
            .run_within_event_loop(
                move |event_loop| create_window(app, attributes, event_loop, window_main, this),
                CallbackTrigger::PollResumed,
            )
            .await?;
        Ok(JoinHandle {
            task,
            executor: self.app.executor.clone(),
            proxy,
        })
    }
}

pub struct JoinHandle<T> {
    task: TaskId,
    executor: Rc<RefCell<ExecutorProxy>>,
    proxy: SharedCallState<T>,
}

impl<T> JoinHandle<T> {
    pub fn terminate(self) {
        self.executor.borrow_mut().terminate(self.task);
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.proxy.poll_unpin(cx)
    }
}

pub struct EventLoopCall<T> {
    proxy: SharedCallState<T>,
}

impl<T> Future for EventLoopCall<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.proxy.poll_unpin(cx)
    }
}
