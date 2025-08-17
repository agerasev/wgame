use alloc::rc::Rc;
use core::{
    cell::RefCell,
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::FutureExt;
use thiserror::Error;
use winit::{error::OsError, window::WindowAttributes};

use crate::{
    Window,
    app::CURRENT_APP,
    executor::{ExecutorProxy, TaskId},
    output::{CallOutput, Terminated},
    proxy::{AppProxy, CallbackTrigger},
    time::{Instant, Timer},
    window::{Suspended, create_window},
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

    /// Panics if called not from app task.
    pub fn current() -> Runtime {
        Runtime::new(
            CURRENT_APP.with_borrow(|proxy| proxy.clone().expect("There's no current runtime")),
        )
    }

    pub fn create_task<F>(&self, future: F) -> TaskHandle<Result<F::Output, Terminated>>
    where
        F: Future<Output: 'static> + 'static,
    {
        let (task_id, output) = self.app.create_task(future);
        TaskHandle {
            task: task_id,
            executor: self.app.executor.clone(),
            output,
        }
    }

    pub fn create_timer(&self, timeout: Duration) -> Timer {
        let timestamp = Instant::now().checked_add(timeout).unwrap();
        self.app.timers.borrow_mut().add(timestamp)
    }

    pub async fn create_windowed_task<T, F>(
        &self,
        attributes: WindowAttributes,
        window_fn: F,
    ) -> Result<TaskHandle<Result<T, Suspended>>, OsError>
    where
        T: 'static,
        F: AsyncFnOnce(Window<'_>) -> T + 'static,
    {
        let app = self.app.clone();
        let (task, output) = self
            .app
            .run_within_event_loop(
                move |event_loop| create_window(app, attributes, event_loop, window_fn),
                CallbackTrigger::PollResumed,
            )
            .await?;
        Ok(TaskHandle {
            task,
            executor: self.app.executor.clone(),
            output,
        })
    }
}

/// Task is **not** terminated on handle drop.
pub struct TaskHandle<T> {
    task: TaskId,
    executor: Rc<RefCell<ExecutorProxy>>,
    output: CallOutput<T>,
}

impl<T> TaskHandle<T> {
    pub fn terminate(self) {
        self.executor.borrow_mut().terminate(self.task);
    }
}

impl<T> Future for TaskHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.output.poll_unpin(cx)
    }
}

pub struct EventLoopCall<T> {
    proxy: CallOutput<T>,
}

impl<T> Future for EventLoopCall<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.proxy.poll_unpin(cx)
    }
}

pub async fn sleep(timeout: Duration) {
    Runtime::current().create_timer(timeout).await;
}

pub fn spawn<F>(future: F) -> TaskHandle<Result<F::Output, Terminated>>
where
    F: Future<Output: 'static> + 'static,
{
    Runtime::current().create_task(future)
}

pub async fn within_window<T, F>(
    attributes: WindowAttributes,
    window_fn: F,
) -> Result<T, WindowError<OsError>>
where
    T: 'static,
    F: AsyncFnOnce(Window<'_>) -> T + 'static,
{
    Runtime::current()
        .create_windowed_task(attributes, window_fn)
        .await?
        .await
        .map_err(|_: Suspended| WindowError::Suspended)
}

#[derive(Debug, Error)]
pub enum WindowError<E: Debug = OsError> {
    #[error("Application suspended")]
    Suspended,
    #[error(transparent)]
    Other(#[from] E),
}
