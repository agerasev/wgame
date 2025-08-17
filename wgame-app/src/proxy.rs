use alloc::{boxed::Box, rc::Rc};
use core::cell::RefCell;

use winit::event_loop::ActiveEventLoop;

use crate::{
    app::{AppState, CallbackContainer},
    executor::{ExecutorProxy, TaskId},
    output::CallOutput,
    time::TimerQueue,
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

#[derive(Clone)]
pub struct AppProxy {
    pub(crate) state: Rc<RefCell<AppState>>,
    pub(crate) executor: Rc<RefCell<ExecutorProxy>>,
    pub(crate) timers: Rc<RefCell<TimerQueue>>,
    pub(crate) callbacks: Rc<RefCell<CallbackContainer>>,
}

impl AppProxy {
    pub fn create_task<F: Future + 'static, E: Default + 'static>(
        &self,
        future: F,
    ) -> (TaskId, CallOutput<Result<F::Output, E>>) {
        let output = CallOutput::default();
        let task_id = self.executor.borrow_mut().spawn(
            {
                let proxy = output.clone();
                async move {
                    let result = future.await;
                    proxy.set_ready(Ok(result));
                }
            },
            output.default_fallible(),
        );
        (task_id, output)
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
}
