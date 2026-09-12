use std::{
    boxed::Box,
    cell::RefCell,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
    time::Duration,
};

use futures::{FutureExt, future::FusedFuture};
use winit::event_loop::ActiveEventLoop;

use crate::{
    app::{AppState, CURRENT, CallbackContainer},
    executor::{ExecutorProxy, TaskId},
    output::{CallOutput, Terminated},
    time::{Instant, Timer, TimerQueue},
};

pub type CallbackObj = Box<dyn FnOnce(&ActiveEventLoop)>;

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub enum CallbackTrigger {
    /// Simply trigger on next event loop poll
    #[default]
    Poll,
    /// Triggner on next event loop poll if application is resumed
    PollResumed,
}

/// Runtime handle
#[derive(Clone)]
pub struct Runtime {
    pub(crate) state: Rc<RefCell<AppState>>,
    pub(crate) executor: Rc<RefCell<ExecutorProxy>>,
    pub(crate) timers: Rc<RefCell<TimerQueue>>,
    pub(crate) callbacks: Rc<RefCell<CallbackContainer>>,
}

impl Runtime {
    /// Panics if called not from app task.
    pub fn current() -> Runtime {
        CURRENT.with_borrow(|proxy| proxy.clone().expect("There's no current runtime"))
    }

    pub fn create_task<F: Future + 'static>(&self, future: F) -> Task<F::Output> {
        let output = CallOutput::default();
        let task_id = self.executor.borrow_mut().spawn(
            {
                let output = output.clone();
                async move {
                    let result = future.await;
                    output.set_ready(Ok(result));
                }
            },
            {
                let output = output.clone();
                move || output.set_ready(Err(Terminated))
            },
        );
        Task {
            task: task_id,
            executor: self.executor.clone(),
            output,
        }
    }

    pub fn run_within_event_loop<T: 'static, F: FnOnce(&ActiveEventLoop) -> T + 'static>(
        &self,
        call: F,
        trigger: CallbackTrigger,
    ) -> CallOutput<T> {
        let output = CallOutput::default();
        let mut callbacks = self.callbacks.borrow_mut();
        let list = match trigger {
            CallbackTrigger::Poll => &mut callbacks.next_poll,
            CallbackTrigger::PollResumed => &mut callbacks.on_resume,
        };
        list.push(Box::new({
            let output = output.clone();
            move |event_loop| {
                let result = call(event_loop);
                output.set_ready(result);
            }
        }));
        output
    }

    pub fn create_timer(&self, timestamp: Instant) -> Timer {
        self.timers.borrow_mut().insert(timestamp)
    }
}

/// Single-consumer result. Dropping the result detaches the task.
/// Clone [`Task::handle`] for cancellation access.
pub struct Task<T> {
    task: TaskId,
    executor: Rc<RefCell<ExecutorProxy>>,
    output: CallOutput<Result<T, Terminated>>,
}

/// Clonable cancellation access without access to the task result.
#[derive(Clone)]
pub struct TaskHandle {
    task: TaskId,
    executor: Rc<RefCell<ExecutorProxy>>,
}
impl TaskHandle {
    pub fn id(&self) -> TaskId {
        self.task
    }
    /// Cancel at the next scheduler boundary. Repeated calls are harmless.
    pub fn terminate(&self) {
        self.executor.borrow_mut().terminate(self.task);
    }
}
/// Task cancelled when its owner leaves scope, including window suspension.
pub struct ScopedTask<T> {
    task: Task<T>,
}
impl<T> ScopedTask<T> {
    pub fn handle(&self) -> TaskHandle {
        self.task.handle()
    }
}
impl<T> Drop for ScopedTask<T> {
    fn drop(&mut self) {
        self.task.terminate();
    }
}
impl<T> Future for ScopedTask<T> {
    type Output = Result<T, Terminated>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.task.poll_unpin(cx)
    }
}
impl<T> FusedFuture for ScopedTask<T> {
    fn is_terminated(&self) -> bool {
        self.task.is_terminated()
    }
}
impl<T> Task<T> {
    /// Opt into cancellation when this handle is dropped.
    pub fn cancel_on_drop(self) -> ScopedTask<T> {
        ScopedTask { task: self }
    }

    pub fn handle(&self) -> TaskHandle {
        TaskHandle {
            task: self.task,
            executor: self.executor.clone(),
        }
    }

    pub fn id(&self) -> TaskId {
        self.task
    }
    pub fn terminate(&self) {
        self.executor.borrow_mut().terminate(self.task);
    }
    pub fn output(&self) -> &CallOutput<Result<T, Terminated>> {
        &self.output
    }
}

impl<T> Future for Task<T> {
    type Output = Result<T, Terminated>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.output.poll_unpin(cx)
    }
}

impl<T> FusedFuture for Task<T> {
    fn is_terminated(&self) -> bool {
        self.output().is_terminated()
    }
}

pub fn sleep_until(timestamp: Instant) -> Timer {
    Runtime::current().create_timer(timestamp)
}

pub fn sleep(timeout: Duration) -> Timer {
    let timestamp = Instant::now().checked_add(timeout).unwrap();
    Runtime::current().create_timer(timestamp)
}

pub fn spawn<F>(future: F) -> Task<F::Output>
where
    F: Future<Output: 'static> + 'static,
{
    Runtime::current().create_task(future)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::Executor;
    #[test]
    fn handle_cancels_result_and_drop_detaches() {
        let mut executor = Executor::with_waker_factory(|_| std::task::Waker::noop().clone());
        let rt = Runtime {
            state: Default::default(),
            executor: executor.proxy(),
            timers: Default::default(),
            callbacks: Default::default(),
        };
        let task = rt.create_task(std::future::pending::<u32>());
        task.handle().clone().terminate();
        assert!(executor.poll().is_ready());
        assert!(matches!(task.output().try_take(), Some(Err(Terminated))));
        let ran = Rc::new(std::cell::Cell::new(false));
        let marker = ran.clone();
        drop(rt.create_task(async move {
            marker.set(true);
        }));
        assert!(executor.poll().is_ready());
        assert!(ran.get());
    }
}
