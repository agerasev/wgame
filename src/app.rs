use std::{
    cell::RefCell,
    rc::Rc,
    task::{Poll, Waker},
};

use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

use crate::executor::{Executor, TaskId};

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
    state: Rc<RefCell<AppState>>,
}

struct AppRuntime {
    state: Rc<RefCell<AppState>>,
    executor: Executor,
}

impl App {
    pub fn new() -> Self {
        let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
        Self {
            event_loop,
            state: Default::default(),
        }
    }

    pub fn shared_state(&self) -> Rc<RefCell<AppState>> {
        self.state.clone()
    }
    pub fn event_loop(&self) -> EventLoopProxy<UserEvent> {
        self.event_loop.create_proxy()
    }

    pub fn run(self, executor: Executor) -> Result<(), EventLoopError> {
        let mut app = AppRuntime {
            state: self.state,
            executor,
        };
        self.event_loop.set_control_flow(ControlFlow::Wait);
        self.event_loop.run_app(&mut app)
    }
}

impl ApplicationHandler<UserEvent> for AppRuntime {
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
