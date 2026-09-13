use crate::{
    Window,
    executor::TaskId,
    output::{CallOutput, Terminated},
    runtime::{CallbackTrigger, Runtime, Task},
    window::create_window,
};
use futures::{FutureExt, future::FusedFuture};
use std::{
    cell::{Cell, RefCell},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};
use thiserror::Error;
use winit::{error::OsError, window::WindowAttributes};

pub fn create_windowed_task<T, F>(
    rt: &Runtime,
    attributes: WindowAttributes,
    window_fn: F,
) -> WindowedTask<T>
where
    T: 'static,
    F: AsyncFnOnce(Window<'_>) -> T + 'static,
{
    let app = rt.clone();
    let cancelled = Rc::new(Cell::new(false));
    let token = cancelled.clone();
    let output = rt.run_within_event_loop(
        move |event_loop| {
            if token.get() {
                Ok(None)
            } else {
                create_window(app, attributes, event_loop, window_fn).map(Some)
            }
        },
        CallbackTrigger::PollResumed,
    );
    WindowedTask(Rc::new(RefCell::new(State {
        stage: Stage::Create(output),
        cancelled,
        finished: false,
        waker: None,
    })))
}
#[derive(Debug, Error)]
pub enum WindowError {
    #[error("Cannot create window: {0}")]
    Creation(OsError),
    #[error("Application suspended")]
    Suspended,
    #[error("Window task terminated")]
    Terminated,
}
/// Single-consumer window task. Dropping it detaches the task.
/// Clone [`Self::handle`] for cancellation access. Cancellation before creation
/// resolves without waiting for OS resume and prevents queued window creation.
/// Suspension cancels the window function and reports [`WindowError::Suspended`];
/// a manually managed caller decides whether to recreate the window.
pub struct WindowedTask<T>(Rc<RefCell<State<T>>>);
/// Clonable cancellation access without access to the result.
pub struct WindowTaskHandle<T>(Rc<RefCell<State<T>>>);
impl<T> Clone for WindowTaskHandle<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T> WindowTaskHandle<T> {
    pub fn terminate(&self) {
        let waker = {
            let mut state = self.0.borrow_mut();
            if state.finished {
                return;
            }
            state.cancelled.set(true);
            match &state.stage {
                Stage::Run(task) => task.terminate(),
                Stage::Create(output) => output.with_ready(|result| {
                    if let Ok(Some(task)) = result {
                        task.terminate();
                    }
                }),
            }
            state.waker.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}
struct State<T> {
    stage: Stage<T>,
    cancelled: Rc<Cell<bool>>,
    finished: bool,
    waker: Option<Waker>,
}
enum Stage<T> {
    Create(CallOutput<Result<Option<Task<T>>, OsError>>),
    Run(Task<T>),
}
impl<T> WindowedTask<T> {
    pub fn id(&self) -> Option<TaskId> {
        match &self.0.borrow().stage {
            Stage::Run(task) => Some(task.id()),
            _ => None,
        }
    }
    pub fn handle(&self) -> WindowTaskHandle<T> {
        WindowTaskHandle(self.0.clone())
    }
    pub fn terminate(&self) {
        self.handle().terminate();
    }
}
impl<T> Future for WindowedTask<T> {
    type Output = Result<T, WindowError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.0.borrow_mut();
        assert!(!state.finished, "Window task result already consumed");
        state.waker = Some(cx.waker().clone());
        if state.cancelled.get() {
            if let Stage::Create(output) = &state.stage
                && let Some(Ok(Some(task))) = output.try_take()
            {
                task.terminate();
            }
            state.finished = true;
            return Poll::Ready(Err(WindowError::Terminated));
        }
        if let Stage::Create(output) = &mut state.stage {
            match output.poll_unpin(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Ok(Some(task))) => state.stage = Stage::Run(task),
                Poll::Ready(Ok(None)) => {
                    state.finished = true;
                    return Poll::Ready(Err(WindowError::Terminated));
                }
                Poll::Ready(Err(err)) => {
                    state.finished = true;
                    return Poll::Ready(Err(WindowError::Creation(err)));
                }
            }
        }
        let Stage::Run(task) = &mut state.stage else {
            unreachable!()
        };
        match task.poll_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => {
                state.finished = true;
                Poll::Ready(result.map_err(|Terminated| WindowError::Suspended))
            }
        }
    }
}
impl<T> FusedFuture for WindowedTask<T> {
    fn is_terminated(&self) -> bool {
        self.0.borrow().finished
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancelling_detached_creation_also_cancels_created_task() {
        let mut executor = crate::executor::Executor::with_waker_factory(|_| Waker::noop().clone());
        let rt = Runtime {
            state: Default::default(),
            executor: executor.proxy(),
            timers: Default::default(),
            callbacks: Default::default(),
        };
        let task = rt.create_task(async {
            panic!("cancelled window ran");
        });
        let output = CallOutput::default();
        output.set_ready(Ok(Some(task)));
        let window = WindowedTask(Rc::new(RefCell::new(State {
            stage: Stage::Create(output),
            cancelled: Rc::new(Cell::new(false)),
            finished: false,
            waker: None,
        })));
        let handle = window.handle();
        drop(window);
        handle.terminate();
        assert!(executor.poll().is_ready());
    }
    #[test]
    fn cancel_before_creation_completes_without_resume() {
        let mut task: WindowedTask<()> = WindowedTask(Rc::new(RefCell::new(State {
            stage: Stage::Create(CallOutput::default()),
            cancelled: Rc::new(Cell::new(false)),
            finished: false,
            waker: None,
        })));
        let mut cx = Context::from_waker(Waker::noop());
        assert!(Pin::new(&mut task).poll(&mut cx).is_pending());
        task.handle().terminate();
        assert!(matches!(
            Pin::new(&mut task).poll(&mut cx),
            Poll::Ready(Err(WindowError::Terminated))
        ));
        assert!(task.is_terminated());
    }
}
