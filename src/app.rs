use std::{
    cell::RefCell,
    rc::Rc,
    task::{Poll, Waker},
};

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

#[derive(Default, Debug)]
pub struct AppState {
    pub window: Option<Window>,
    pub redraw_waker: Option<Waker>,
    pub redraw_requested: bool,
    pub close_requested: bool,
}

pub struct App {
    event_loop: EventLoop<UserEvent>,
    executor: Executor,
    state: Rc<RefCell<AppState>>,
}

#[derive(Clone)]
pub struct AppProxy {
    pub state: Rc<RefCell<AppState>>,
    pub executor: Rc<RefCell<ExecutorProxy>>,
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
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.state.borrow_mut().window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let mut state = self.state.borrow_mut();
        match event {
            WindowEvent::CloseRequested => {
                state.close_requested = true;
                if let Some(waker) = state.redraw_waker.take() {
                    waker.wake()
                }
            }
            WindowEvent::RedrawRequested => {
                state.redraw_requested = true;
                if let Some(waker) = state.redraw_waker.take() {
                    waker.wake()
                }
            }
            _ => (),
        }
        drop(state);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        self.executor.add_task_to_poll(event.task_id);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
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
