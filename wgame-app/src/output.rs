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

/// Shared single-consumption output. Clones share one result and one waiter.
/// Use one consumer; this is not a broadcast channel. Polling after consumption
/// or completing the output twice is a programming error and panics.
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
    pub(crate) fn with_ready(&self, f: impl FnOnce(&T)) {
        let state = self.0.take();
        if let State::Ready(value) = &state {
            f(value);
        }
        self.0.set(state);
    }
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
        let previous = self.0.replace(State::Ready(value));
        match previous {
            State::Pending(Some(waker)) => waker.wake(),
            State::Pending(None) => (),
            other => {
                self.0.set(other);
                panic!("Call output completed more than once");
            }
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct Terminated;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completion_before_poll_is_consumed_once() {
        let output = CallOutput::default();
        output.set_ready(42);
        assert!(!output.is_terminated());
        assert_eq!(output.try_take(), Some(42));
        assert!(output.is_terminated());
    }
    #[test]
    #[should_panic(expected = "completed more than once")]
    fn double_completion_is_rejected() {
        let output = CallOutput::default();
        output.set_ready(1);
        output.set_ready(2);
    }
}
