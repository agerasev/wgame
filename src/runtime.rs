use std::{
    cell::RefCell,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

use winit::{event_loop::ActiveEventLoop, window::WindowAttributes};

use crate::{
    Window,
    app::AppProxy,
    executor::{TaskId, Timer},
    surface::SurfaceBuilder,
    window::CreateError,
};

/// Handle to underlying async runtime.
#[derive(Clone)]
pub struct Runtime {
    app: AppProxy,
}

impl Runtime {
    pub(crate) fn new(app: AppProxy) -> Self {
        Self { app }
    }

    pub fn spawn<T: 'static, F: Future<Output = T> + 'static>(&self, future: F) -> JoinHandle<T> {
        let proxy = Rc::new(RefCell::new(CallProxy::default()));
        let task_id = self.app.executor.borrow_mut().spawn({
            let proxy = proxy.clone();
            async move {
                let output = future.await;

                let mut proxy = proxy.borrow_mut();
                proxy.output = Some(output);
                if let Some(waker) = proxy.waker.take() {
                    waker.wake();
                }
            }
        });
        JoinHandle {
            _task_id: task_id,
            proxy,
        }
    }

    pub fn sleep(&self, timeout: Duration) -> Sleep {
        let timestamp = Instant::now().checked_add(timeout).unwrap();
        let timer = self.app.executor.borrow_mut().add_timer(timestamp);
        Sleep { timer }
    }

    pub fn with_event_loop<T: 'static, F: FnOnce(&ActiveEventLoop) -> T + 'static>(
        &self,
        call: F,
    ) -> EventLoopCall<T> {
        let proxy = Rc::new(RefCell::new(CallProxy::default()));
        self.app.executor.borrow_mut().add_loop_call({
            let proxy = proxy.clone();
            move |event_loop: &ActiveEventLoop| {
                let output = call(event_loop);

                let mut proxy = proxy.borrow_mut();
                proxy.output = Some(output);
                if let Some(waker) = proxy.waker.take() {
                    waker.wake();
                }
            }
        });
        EventLoopCall { proxy }
    }

    pub async fn create_window<S: SurfaceBuilder>(
        &self,
        attributes: WindowAttributes,
        builder: S,
    ) -> Result<Window<S>, CreateError<S::Error>> {
        let mut window = Window::new(self.app.clone(), attributes, builder);
        window.create().await?;
        Ok(window)
    }
}

struct CallProxy<T> {
    output: Option<T>,
    waker: Option<Waker>,
}

impl<T> Default for CallProxy<T> {
    fn default() -> Self {
        Self {
            output: None,
            waker: None,
        }
    }
}

pub struct JoinHandle<T> {
    _task_id: TaskId,
    proxy: Rc<RefCell<CallProxy<T>>>,
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut proxy = self.proxy.borrow_mut();
        if let Some(output) = proxy.output.take() {
            Poll::Ready(output)
        } else {
            proxy.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub struct Sleep {
    timer: Timer,
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.timer.timestamp {
            Poll::Ready(())
        } else {
            self.timer.waker.set(Some(cx.waker().clone()));
            Poll::Pending
        }
    }
}

pub struct EventLoopCall<T> {
    proxy: Rc<RefCell<CallProxy<T>>>,
}

impl<T> Future for EventLoopCall<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut proxy = self.proxy.borrow_mut();
        if let Some(output) = proxy.output.take() {
            Poll::Ready(output)
        } else {
            proxy.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
