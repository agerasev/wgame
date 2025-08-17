use alloc::rc::Rc;
use core::{
    cell::Cell,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use futures::future::FusedFuture;

enum State<T> {
    Pending(Option<Waker>),
    Ready(T),
    Completed,
}

impl<T> Default for State<T> {
    fn default() -> Self {
        Self::Pending(None)
    }
}

pub struct CallOutput<T>(Rc<Cell<State<T>>>);

impl<T> Clone for CallOutput<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Default for CallOutput<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> Future for CallOutput<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.0.replace(State::Completed) {
            State::Pending(_) => {
                self.0.set(State::Pending(Some(cx.waker().clone())));
                Poll::Pending
            }
            State::Ready(value) => Poll::Ready(value),
            State::Completed => panic!("Call state is completed already"),
        }
    }
}

impl<T> FusedFuture for CallOutput<T> {
    fn is_terminated(&self) -> bool {
        match self.0.replace(State::Completed) {
            State::Completed => true,
            other => {
                self.0.set(other);
                false
            }
        }
    }
}

impl<T> CallOutput<T> {
    pub fn set_ready(&self, value: T) {
        if let State::Pending(Some(waker)) = self.0.take() {
            waker.wake();
        }
        self.0.set(State::Ready(value));
    }
}
impl<T: 'static, E: Default + 'static> CallOutput<Result<T, E>> {
    pub fn default_fallible(&self) -> Rc<dyn DefaultFallible> {
        self.0.clone() as _
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct Terminated;

pub trait DefaultFallible {
    fn set_failed(&self);
}

impl<T, E: Default> DefaultFallible for Cell<State<Result<T, E>>> {
    fn set_failed(&self) {
        if let State::Pending(Some(waker)) = self.take() {
            waker.wake();
        }
        self.set(State::Ready(Err(E::default())));
    }
}
