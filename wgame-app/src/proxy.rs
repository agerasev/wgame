use alloc::{boxed::Box, rc::Rc};
use core::{
    cell::{Cell, RefCell},
    pin::Pin,
    task::{Context, Poll, Waker},
};

use futures::future::FusedFuture;
use winit::event_loop::ActiveEventLoop;

use crate::{
    app::{AppState, CallbackContainer},
    executor::{ExecutorProxy, TaskId},
    timer::TimerQueue,
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

pub enum CallState<T> {
    Pending(Option<Waker>),
    Ready(T),
    Done,
}

impl<T> Default for CallState<T> {
    fn default() -> Self {
        Self::Pending(None)
    }
}

pub struct SharedCallState<T>(Rc<Cell<CallState<T>>>);

impl<T> Clone for SharedCallState<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Default for SharedCallState<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> Future for SharedCallState<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.0.replace(CallState::Done) {
            CallState::Pending(_) => {
                self.0.set(CallState::Pending(Some(cx.waker().clone())));
                Poll::Pending
            }
            CallState::Ready(value) => Poll::Ready(value),
            CallState::Done => panic!("Call state is done already"),
        }
    }
}

impl<T> FusedFuture for SharedCallState<T> {
    fn is_terminated(&self) -> bool {
        match self.0.replace(CallState::Done) {
            CallState::Done => true,
            other => {
                self.0.set(other);
                false
            }
        }
    }
}

impl<T> SharedCallState<T> {
    pub fn set_ready(&self, value: T) {
        if let CallState::Pending(Some(waker)) = self.0.take() {
            waker.wake();
        }
        self.0.set(CallState::Ready(value));
    }
}

impl AppProxy {
    pub fn create_task<F: Future + 'static>(
        &self,
        future: F,
    ) -> (TaskId, SharedCallState<F::Output>) {
        let proxy = SharedCallState::default();
        let task_id = self.executor.borrow_mut().spawn({
            let proxy = proxy.clone();
            async move {
                let result = future.await;
                proxy.set_ready(result);
            }
        });
        (task_id, proxy)
    }

    pub fn run_within_event_loop<T: 'static, F: FnOnce(&ActiveEventLoop) -> T + 'static>(
        &self,
        call: F,
        trigger: CallbackTrigger,
    ) -> SharedCallState<T> {
        let proxy = SharedCallState::default();
        let mut callbacks = self.callbacks.borrow_mut();
        let list = match trigger {
            CallbackTrigger::Poll => &mut callbacks.next_poll,
            CallbackTrigger::PollResumed => &mut callbacks.on_resume,
        };
        list.push(Box::new({
            let proxy = proxy.clone();
            move |event_loop| {
                let result = call(event_loop);
                proxy.set_ready(result);
            }
        }));
        proxy
    }
}
