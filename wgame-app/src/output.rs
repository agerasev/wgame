use std::{
    cell::Cell,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use futures::future::FusedFuture;

enum State<T> {
    Pending(Option<Waker>),
    Ready(T),
    Taken,
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

impl<T> CallOutput<T> {
    pub fn try_take(&self) -> Option<T> {
        match self.0.replace(State::Taken) {
            State::Pending(waker) => {
                self.0.set(State::Pending(waker));
                None
            }
            State::Ready(value) => Some(value),
            State::Taken => panic!("Call output is already taken"),
        }
    }
}

impl<T> Future for CallOutput<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.0.replace(State::Taken) {
            State::Pending(_) => {
                self.0.set(State::Pending(Some(cx.waker().clone())));
                Poll::Pending
            }
            State::Ready(value) => Poll::Ready(value),
            State::Taken => panic!("Call output is already taken"),
        }
    }
}

impl<T> FusedFuture for CallOutput<T> {
    fn is_terminated(&self) -> bool {
        match self.0.replace(State::Taken) {
            State::Taken => true,
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

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct Terminated;
