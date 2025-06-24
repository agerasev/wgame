use std::{
    cell::RefCell,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures::FutureExt;
use winit::{error::OsError, window::WindowAttributes};

use crate::{
    Window,
    executor::{ExecutorProxy, TaskId},
    proxy::{AppProxy, CallbackTrigger, SharedCallState},
    timer::Timer,
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

    pub fn spawn<T: 'static, F: Future<Output = T> + 'static>(&self, future: F) -> JoinHandle<T> {
        let (task_id, proxy) = self.app.create_task(future);
        JoinHandle {
            task: task_id,
            executor: self.app.executor.clone(),
            proxy,
        }
    }

    pub fn sleep(&self, timeout: Duration) -> Timer {
        let timestamp = Instant::now().checked_add(timeout).unwrap();
        self.app.timers.borrow_mut().add(timestamp)
    }

    pub async fn create_window<T: 'static, F: AsyncFnOnce(&mut Window) -> T + 'static>(
        &self,
        attributes: WindowAttributes,
        window_main: F,
    ) -> Result<JoinHandle<T>, OsError> {
        let app = self.app.clone();
        let (task, proxy) = self
            .app
            .with_event_loop(
                move |event_loop| {
                    let raw = event_loop.create_window(attributes)?;
                    let id = raw.id();
                    let mut window = Window::new(raw, app.clone());
                    let state = Rc::downgrade(window.state());
                    let (task, proxy) =
                        app.create_task(async move { window_main(&mut window).await });
                    app.state.borrow_mut().insert_window(id, task, state);
                    Ok((task, proxy))
                },
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
