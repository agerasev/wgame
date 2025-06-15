use std::{
    cell::RefCell,
    rc::Rc,
    task::{Poll, Waker},
};

use fxhash::FxHashMap as HashMap;
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::executor::{Executor, ExecutorProxy, TaskId};

#[derive(Debug)]
pub struct UserEvent {
    pub task_id: TaskId,
}

pub struct WindowState {
    pub window: Window,
    pub waker: Option<Waker>,
    pub redraw_requested: bool,
    pub close_requested: bool,
}

impl WindowState {
    pub fn new(window: Window) -> Self {
        Self {
            window,
            waker: None,
            redraw_requested: false,
            close_requested: false,
        }
    }
}

#[derive(Default)]
pub struct AppState {
    pub windows: HashMap<WindowId, WindowState>,
}

pub struct App {
    event_loop: EventLoop<UserEvent>,
    executor: Executor,
    state: Rc<RefCell<AppState>>,
}

#[derive(Clone)]
pub struct AppProxy {
    pub(crate) state: Rc<RefCell<AppState>>,
    pub(crate) executor: Rc<RefCell<ExecutorProxy>>,
}

struct AppHandler {
    state: Rc<RefCell<AppState>>,
    executor: Executor,
}

impl App {
    pub fn new() -> Result<Self, EventLoopError> {
        let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
        let executor = Executor::new(event_loop.create_proxy());
        Ok(Self {
            event_loop,
            executor,
            state: Default::default(),
        })
    }

    pub fn proxy(&self) -> AppProxy {
        AppProxy {
            state: self.state.clone(),
            executor: self.executor.proxy(),
        }
    }

    pub fn run(self) -> Result<(), EventLoopError> {
        let mut app = AppHandler {
            state: self.state,
            executor: self.executor,
        };
        self.event_loop.set_control_flow(ControlFlow::Wait);
        self.event_loop.run_app(&mut app)
    }
}

impl ApplicationHandler<UserEvent> for AppHandler {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        log::trace!("resumed");
        // FIXME: Re-create windows
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        log::trace!("window_event {id:?}: {event:?}");

        let mut state = self.state.borrow_mut();
        let window = match state.windows.get_mut(&id) {
            Some(window) => window,
            None => {
                if event != WindowEvent::Destroyed {
                    log::error!("No such window {id:?}: {event:?}");
                }
                return;
            }
        };
        match event {
            WindowEvent::CloseRequested => {
                window.close_requested = true;
                if let Some(waker) = window.waker.take() {
                    waker.wake()
                }
            }
            WindowEvent::RedrawRequested => {
                window.redraw_requested = true;
                if let Some(waker) = window.waker.take() {
                    waker.wake()
                }
            }
            _ => (),
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        log::trace!("user_event: {event:?}");
        self.executor.add_task_to_poll(event.task_id);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        log::trace!("about_to_wait");
        match self.executor.poll(event_loop) {
            Poll::Pending => (),
            Poll::Ready(()) => event_loop.exit(),
        }
    }
}

impl AppProxy {
    pub fn spawn<F: Future<Output = ()> + 'static>(&mut self, future: F) {
        self.executor.borrow_mut().spawn(future);
    }
}
