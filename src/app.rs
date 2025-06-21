use std::{
    cell::{RefCell, RefMut},
    collections::{VecDeque, hash_map::Entry},
    rc::{Rc, Weak},
    task::{Poll, Waker},
};

use fxhash::FxHashMap as HashMap;
use winit::{
    application::ApplicationHandler,
    dpi::Size,
    error::{EventLoopError, OsError},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

use crate::{
    Runtime,
    executor::{Executor, ExecutorProxy, TaskId},
};

#[derive(Debug)]
pub struct UserEvent {
    pub task_id: TaskId,
}

const EVENTS_CAPACITY: usize = 0x1000;

pub struct WindowState {
    pub attributes: WindowAttributes,
    pub dynamic: Option<WindowDynamicState>,
    pub waker: Option<Waker>,
    pub events: VecDeque<WindowEvent>,
    pub close_requested: bool,
}

pub struct WindowDynamicState {
    pub window: Window,
    pub redraw_requested: bool,
}

impl WindowState {
    fn new(attributes: WindowAttributes) -> Self {
        Self {
            attributes,
            dynamic: None,
            waker: None,
            events: VecDeque::new(),
            close_requested: false,
        }
    }

    fn create_window_if_not_exist(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<WindowId, OsError> {
        if let Some(dynamic) = &self.dynamic {
            return Ok(dynamic.window.id());
        }
        let window = event_loop.create_window(self.attributes.clone())?;
        let id = window.id();
        self.dynamic = Some(WindowDynamicState {
            window,
            redraw_requested: false,
        });
        Ok(id)
    }

    fn push_event(&mut self, event: WindowEvent) {
        match &event {
            WindowEvent::CloseRequested => {
                self.close_requested = true;
            }
            WindowEvent::RedrawRequested => {
                if let Some(dynamic) = &mut self.dynamic {
                    dynamic.redraw_requested = true;
                }
            }
            WindowEvent::Resized(size) => {
                self.attributes.inner_size = Some(Size::Physical(*size));
            }
            _ => (),
        }

        while self.events.len() >= EVENTS_CAPACITY {
            self.events.pop_front();
        }
        self.events.push_back(event);
        if let Some(waker) = self.waker.take() {
            waker.wake()
        }
    }
}

#[derive(Default)]
pub struct AppState {
    active: bool,
    resumed: HashMap<WindowId, Weak<RefCell<WindowState>>>,
    suspended: Vec<Weak<RefCell<WindowState>>>,
}

impl AppState {
    pub fn create_window_state(
        &mut self,
        attributes: WindowAttributes,
    ) -> Rc<RefCell<WindowState>> {
        let state = Rc::new(RefCell::new(WindowState::new(attributes)));
        self.suspended.push(Rc::downgrade(&state));
        log::debug!("A new window state created");
        state
    }

    pub fn try_create_and_insert_window_if_not_exist(
        &mut self,
        state: &Rc<RefCell<WindowState>>,
        event_loop: &ActiveEventLoop,
    ) -> Result<bool, OsError> {
        Ok(if self.active {
            let mut window = state.borrow_mut();
            let id = window.create_window_if_not_exist(event_loop)?;
            log::debug!("Window {id:?} created");
            if let Entry::Vacant(entry) = self.resumed.entry(id) {
                let weak = Rc::downgrade(state);
                self.suspended.retain(|w| !w.ptr_eq(&weak));
                entry.insert(weak);
                log::debug!("Window {id:?} resumed");
            }
            true
        } else {
            false
        })
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

pub struct App {
    event_loop: EventLoop<UserEvent>,
    executor: Executor,
    state: Rc<RefCell<AppState>>,
}

#[derive(Clone)]
pub(crate) struct AppProxy {
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

    pub(crate) fn proxy(&self) -> AppProxy {
        AppProxy {
            state: self.state.clone(),
            executor: self.executor.proxy(),
        }
    }

    pub fn runtime(&self) -> Runtime {
        Runtime::new(self.proxy())
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
        log::debug!("resumed");
        let mut state = self.state.borrow_mut();
        state.active = true;
        state.suspended.retain(|window| {
            if let Some(window) = &window.upgrade() {
                if let Some(waker) = window.borrow_mut().waker.take() {
                    waker.wake();
                }
                true
            } else {
                log::warn!("Window dropped but not removed from the app");
                false
            }
        })
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        log::debug!("suspended");
        let mut state = self.state.borrow_mut();
        state.active = false;
        let (mut suspended, mut resumed) =
            RefMut::map_split(state, |state| (&mut state.suspended, &mut state.resumed));
        for (_, window) in resumed.drain() {
            match window.upgrade() {
                Some(window) => {
                    // Drow window and surface
                    window.borrow_mut().dynamic = None;
                    suspended.push(Rc::downgrade(&window));
                }
                None => log::warn!("Window dropped but not removed from the app"),
            }
        }
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        log::trace!("window_event {id:?}: {event:?}");

        let mut state = self.state.borrow_mut();
        match state.resumed.entry(id) {
            Entry::Occupied(mut entry) => {
                if let Some(window) = entry.get_mut().upgrade() {
                    let destroyed = event == WindowEvent::Destroyed;
                    window.borrow_mut().push_event(event);
                    if destroyed {
                        entry.remove();
                    }
                } else {
                    log::warn!("Window dropped but not removed from the app");
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
        match self.executor.poll(event_loop) {
            Poll::Pending => (),
            Poll::Ready(()) => event_loop.exit(),
        }
    }
}
