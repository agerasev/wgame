use std::{
    cell::{RefCell, RefMut},
    collections::{VecDeque, hash_map::Entry},
    hash::{Hash, Hasher},
    ops::Deref,
    rc::{Rc, Weak},
    task::{Poll, Waker},
};

use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
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
    surface::Surface,
};

#[derive(Debug)]
pub struct UserEvent {
    pub task_id: TaskId,
}

const EVENTS_CAPACITY: usize = 0x1000;

pub struct WindowState {
    pub attributes: WindowAttributes,
    pub actual: Option<ActualWindowState>,
    pub waker: Option<Waker>,
    pub events: VecDeque<WindowEvent>,
    pub close_requested: bool,
}

pub struct ActualWindowState {
    pub window: Window,
    pub surface: Option<Box<dyn Surface>>,
    pub redraw_requested: bool,
}

impl WindowState {
    fn new(attributes: WindowAttributes) -> Self {
        Self {
            attributes,
            actual: None,
            waker: None,
            events: VecDeque::new(),
            close_requested: false,
        }
    }

    fn create_actual(&mut self, event_loop: &ActiveEventLoop) -> Result<WindowId, OsError> {
        if let Some(actual) = &self.actual {
            return Ok(actual.window.id());
        }
        let window = event_loop.create_window(self.attributes.clone())?;
        let id = window.id();
        self.actual = Some(ActualWindowState {
            window,
            surface: None,
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
                if let Some(actual) = &mut self.actual {
                    actual.redraw_requested = true;
                }
            }
            WindowEvent::Resized(size) => {
                self.attributes.inner_size = Some(Size::Physical(*size));
                if let Some(actual) = &mut self.actual {
                    if let Some(surface) = &mut actual.surface {
                        surface.resize(*size);
                    }
                }
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

#[derive(Clone, Default)]
struct WindowHandle(Weak<RefCell<WindowState>>);

impl WindowHandle {
    fn new(state: &Rc<RefCell<WindowState>>) -> Self {
        Self(Rc::downgrade(state))
    }

    fn upgrade(&self) -> Option<Rc<RefCell<WindowState>>> {
        match self.0.upgrade() {
            Some(state) => Some(state),
            None => {
                log::warn!("Window dropped but not removed from the app");
                None
            }
        }
    }
}

impl Deref for WindowHandle {
    type Target = Weak<RefCell<WindowState>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for WindowHandle {
    fn eq(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }
}

impl Eq for WindowHandle {}

impl Hash for WindowHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ptr().hash(state)
    }
}

#[derive(Default)]
struct WindowContainer {
    resumed: HashMap<WindowId, WindowHandle>,
    suspended: HashSet<WindowHandle>,
}

#[derive(Default)]
pub struct AppState {
    active: bool,
    windows: WindowContainer,
}

impl AppState {
    pub fn new_window_state(&mut self, attributes: WindowAttributes) -> Rc<RefCell<WindowState>> {
        let state = Rc::new(RefCell::new(WindowState::new(attributes)));
        self.windows.suspended.insert(WindowHandle::new(&state));
        log::debug!("A new window state created");
        state
    }

    pub fn create_actual_window(
        &mut self,
        state: &Rc<RefCell<WindowState>>,
        event_loop: &ActiveEventLoop,
    ) -> Result<bool, OsError> {
        Ok(if self.active {
            let mut window = state.borrow_mut();
            let id = window.create_actual(event_loop)?;
            log::debug!("Window {id:?} created");
            if let Entry::Vacant(entry) = self.windows.resumed.entry(id) {
                let handle = WindowHandle::new(state);
                if !self.windows.suspended.remove(&handle) {
                    log::warn!("Cannot remove window from suspended: not found");
                }
                entry.insert(handle);
            }
            true
        } else {
            false
        })
    }

    pub fn remove_window(&mut self, state: &Rc<RefCell<WindowState>>) {
        match state.borrow().actual.as_ref().map(|a| a.window.id()) {
            Some(id) => {
                if self.windows.resumed.remove(&id).is_none() {
                    log::warn!("Cannot remove window from resumed: {id:?} not found");
                }
            }
            None => {
                let handle = WindowHandle::new(state);
                if !self.windows.suspended.remove(&handle) {
                    log::warn!("Cannot remove window from suspended: not found");
                }
            }
        }
    }

    /// Whether app is suspended or resumed
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

        // Poll tasks before running the event loop
        app.executor.poll_tasks();

        self.event_loop.set_control_flow(ControlFlow::Wait);
        self.event_loop.run_app(&mut app)
    }
}

impl ApplicationHandler<UserEvent> for AppHandler {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        log::debug!("resumed");
        let mut state = self.state.borrow_mut();
        state.active = true;
        state.windows.suspended.retain(|window| {
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
        let (mut suspended, mut resumed) = RefMut::map_split(state, |s| {
            (&mut s.windows.suspended, &mut s.windows.resumed)
        });
        for (_, window) in resumed.drain() {
            if let Some(window) = window.upgrade() {
                // Drow actual window
                window.borrow_mut().actual = None;
                suspended.insert(WindowHandle::new(&window));
            }
        }
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        log::trace!("window_event {id:?}: {event:?}");

        let mut state = self.state.borrow_mut();
        match state.windows.resumed.entry(id) {
            Entry::Occupied(mut entry) => {
                if let Some(window) = entry.get_mut().upgrade() {
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
        match self.executor.poll(event_loop) {
            Poll::Pending => (),
            Poll::Ready(()) => event_loop.exit(),
        }
    }
}
