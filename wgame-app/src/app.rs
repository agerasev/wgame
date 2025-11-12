use std::{
    cell::RefCell,
    rc::{Rc, Weak},
    task::Poll,
    thread_local,
};

use hashbrown::hash_map::{Entry, HashMap};
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::WindowId,
};

use crate::{
    executor::{Executor, TaskId},
    runtime::{CallbackObj, Runtime},
    time::TimerQueue,
    window::WindowState,
};

thread_local! {
    pub static CURRENT: RefCell<Option<Runtime>> = const { RefCell::new(None) };
}

#[derive(Debug)]
pub struct UserEvent {
    pub task_id: TaskId,
}

#[derive(Default)]
pub struct CallbackContainer {
    pub next_poll: Vec<CallbackObj>,
    pub on_resume: Vec<CallbackObj>,
}

#[derive(Default)]
pub struct AppState {
    resumed: bool,
    windows: HashMap<WindowId, (TaskId, Weak<RefCell<WindowState>>)>,
}

impl AppState {
    pub fn insert_window(&mut self, id: WindowId, task: TaskId, state: Weak<RefCell<WindowState>>) {
        if let Entry::Vacant(entry) = self.windows.entry(id) {
            entry.insert((task, state));
        } else {
            log::error!("Window {id:?} already registered");
        }
    }

    pub fn remove_window(&mut self, id: WindowId) {
        if let Some((_task, window)) = self.windows.remove(&id) {
            if let Some(state) = window.upgrade() {
                state.borrow_mut().terminate();
            }
        } else {
            log::warn!("Cannot remove window from resumed: {id:?} not found");
        }
    }

    pub fn is_resumed(&self) -> bool {
        self.resumed
    }
}

pub struct App {
    event_loop: EventLoop<UserEvent>,
    executor: Executor,
    state: Rc<RefCell<AppState>>,
    timers: Rc<RefCell<TimerQueue>>,
    callbacks: Rc<RefCell<CallbackContainer>>,
}

struct AppHandler {
    state: Rc<RefCell<AppState>>,
    executor: Executor,
    timers: Rc<RefCell<TimerQueue>>,
    callbacks: Rc<RefCell<CallbackContainer>>,
    call_buffer: Vec<CallbackObj>,
}

impl App {
    pub fn new() -> Result<Self, EventLoopError> {
        let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
        let executor = Executor::new(event_loop.create_proxy());
        Ok(Self {
            event_loop,
            executor,
            state: Default::default(),
            timers: Default::default(),
            callbacks: Default::default(),
        })
    }

    pub fn runtime(&self) -> Runtime {
        Runtime {
            executor: self.executor.proxy(),
            state: self.state.clone(),
            timers: self.timers.clone(),
            callbacks: self.callbacks.clone(),
        }
    }

    pub fn run(self) -> Result<(), EventLoopError> {
        assert!(CURRENT.replace(Some(self.runtime())).is_none());

        let mut app = AppHandler {
            state: self.state,
            executor: self.executor,
            timers: self.timers,
            callbacks: self.callbacks,
            call_buffer: Vec::new(),
        };

        // Poll tasks before running the event loop
        let result = if app.executor.poll().is_ready() {
            Ok(())
        } else {
            self.event_loop.set_control_flow(ControlFlow::Wait);
            self.event_loop.run_app(&mut app)
        };

        assert!(CURRENT.replace(None).is_some());

        result
    }
}

impl ApplicationHandler<UserEvent> for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::debug!("resumed");
        self.call_buffer
            .append(&mut self.callbacks.borrow_mut().on_resume);
        for call in self.call_buffer.drain(..) {
            call(event_loop);
        }
        self.state.borrow_mut().resumed = true;
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        log::debug!("suspended");
        let mut state = self.state.borrow_mut();
        state.resumed = false;
        for (id, (task, _window)) in state.windows.drain() {
            self.state.borrow_mut().remove_window(id);
            self.executor.terminate_task(task);
        }
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        log::trace!("window_event {id:?}: {event:?}");
        match self.state.borrow_mut().windows.entry(id) {
            Entry::Occupied(mut entry) => {
                if let Some(window) = entry.get_mut().1.upgrade() {
                    if event != WindowEvent::Destroyed {
                        window.borrow_mut().push_event(event);
                    } else {
                        entry.remove();
                    }
                } else {
                    entry.remove();
                }
            }
            Entry::Vacant(_) => {
                if event != WindowEvent::Destroyed {
                    log::warn!("No such window {id:?}: {event:?}");
                }
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        log::trace!("user_event: {event:?}");
        self.executor.add_task_to_poll(event.task_id);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        log::trace!("about_to_wait");

        match self.executor.poll() {
            Poll::Pending => (),
            Poll::Ready(()) => event_loop.exit(),
        }

        let mut callbacks = self.callbacks.borrow_mut();
        self.call_buffer.append(&mut callbacks.next_poll);
        if self.state.borrow().is_resumed() {
            self.call_buffer.append(&mut callbacks.on_resume);
        }
        for call in self.call_buffer.drain(..) {
            call(event_loop);
        }

        let next_poll = self.timers.borrow_mut().poll();
        event_loop.set_control_flow(next_poll);
    }
}
