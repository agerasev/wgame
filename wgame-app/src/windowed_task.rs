use alloc::rc::Rc;
use core::{
    cell::{RefCell, RefMut},
    pin::Pin,
    task::{Context, Poll},
};
use futures::{FutureExt, future::FusedFuture};

use thiserror::Error;
use winit::{error::OsError, window::WindowAttributes};

use crate::{
    Window,
    executor::TaskId,
    output::{CallOutput, Terminated},
    runtime::{CallbackTrigger, Runtime, Task},
    window::create_window,
};

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
    let output = rt.run_within_event_loop(
        move |event_loop| create_window(app, attributes, event_loop, window_fn),
        CallbackTrigger::PollResumed,
    );
    WindowedTask(Rc::new(RefCell::new(State {
        stage: Stage::Create(output),
        terminated: false,
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

pub struct WindowedTask<T>(Rc<RefCell<State<T>>>);

impl<T> Clone for WindowedTask<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

struct State<T> {
    stage: Stage<T>,
    terminated: bool,
}

enum Stage<T> {
    Create(CallOutput<Result<Task<T>, OsError>>),
    Run(Task<T>),
}

impl<T> WindowedTask<T> {
    pub fn id(&self) -> Option<TaskId> {
        match &self.0.borrow().stage {
            Stage::Run(task) => Some(task.id()),
            _ => None,
        }
    }
    pub fn terminate(&self) {
        let (stage, mut terminated) = RefMut::map_split(self.0.borrow_mut(), |state| {
            (&mut state.stage, &mut state.terminated)
        });
        *terminated = true;
        if let Stage::Run(task) = &*stage {
            task.terminate();
        }
    }
}

impl<T> Future for WindowedTask<T> {
    type Output = Result<T, WindowError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.0.borrow_mut();
        let task = match &mut state.stage {
            Stage::Create(output) => match output.poll_unpin(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(result) => match result {
                    Ok(task) => Some(task),
                    Err(err) => return Poll::Ready(Err(WindowError::Creation(err))),
                },
            },
            _ => None,
        };
        if let Some(task) = task {
            if state.terminated {
                task.terminate();
            }
            state.stage = Stage::Run(task);
        }
        match &mut state.stage {
            Stage::Create(_) => unreachable!(),
            Stage::Run(task) => match task.poll_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(result) => Poll::Ready(match result {
                    Ok(x) => Ok(x),
                    Err(Terminated) => {
                        if state.terminated {
                            Err(WindowError::Terminated)
                        } else {
                            Err(WindowError::Suspended)
                        }
                    }
                }),
            },
        }
    }
}

impl<T> FusedFuture for WindowedTask<T> {
    fn is_terminated(&self) -> bool {
        match &self.0.borrow().stage {
            Stage::Create(output) => output.is_terminated(),
            Stage::Run(task) => task.is_terminated(),
        }
    }
}
