use std::{
    cell::RefCell,
    collections::hash_map::Entry,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use futures::task::{ArcWake, waker};
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use winit::event_loop::EventLoopProxy;

type FutureObj = Pin<Box<dyn Future<Output = ()>>>;

use crate::app::UserEvent;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct TaskId(u64);

impl TaskId {
    fn get_and_inc(&mut self) -> TaskId {
        let id = *self;
        self.0 = self.0.wrapping_add(1);
        id
    }
}

struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
    waker: Waker,
}

struct TaskData {
    id: TaskId,
    event_loop: EventLoopProxy<UserEvent>,
}

impl ArcWake for TaskData {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        if arc_self
            .event_loop
            .send_event(UserEvent {
                task_id: arc_self.id,
            })
            .is_err()
        {
            panic!("Event loop closed");
        }
    }
}

impl Task {
    fn new(
        id: TaskId,
        event_loop: EventLoopProxy<UserEvent>,
        future: Pin<Box<dyn Future<Output = ()>>>,
    ) -> Self {
        let data = Arc::new(TaskData { id, event_loop });
        Self {
            future,
            waker: waker(data),
        }
    }

    fn poll(&mut self) -> Poll<()> {
        let mut cx = Context::from_waker(&self.waker);
        match self.future.as_mut().poll(&mut cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(()) => Poll::Ready(()),
        }
    }
}

pub struct Executor {
    event_loop: EventLoopProxy<UserEvent>,
    tasks: HashMap<TaskId, Task>,
    tasks_to_poll: HashSet<TaskId>,
    proxy: Rc<RefCell<ExecutorProxy>>,
}

#[derive(Default)]
pub struct ExecutorProxy {
    task_counter: TaskId,
    new_tasks: Vec<(TaskId, FutureObj)>,
}

impl Executor {
    pub fn new(event_loop: EventLoopProxy<UserEvent>) -> Self {
        Self {
            event_loop,
            tasks: HashMap::default(),
            tasks_to_poll: HashSet::default(),
            proxy: Rc::new(RefCell::new(ExecutorProxy::default())),
        }
    }

    pub fn proxy(&self) -> Rc<RefCell<ExecutorProxy>> {
        self.proxy.clone()
    }

    fn sync(&mut self) {
        let mut proxy = self.proxy.borrow_mut();

        for (id, future) in proxy.new_tasks.drain(..) {
            let task = Task::new(id, self.event_loop.clone(), future);
            assert!(self.tasks.insert(id, task).is_none());
            self.tasks_to_poll.insert(id);
            log::trace!("task spawned: {id:?}");
        }
    }

    pub fn add_task_to_poll(&mut self, task_id: TaskId) {
        self.tasks_to_poll.insert(task_id);
    }

    pub fn poll(&mut self) -> Poll<()> {
        log::trace!("poll");

        self.sync();

        while !self.tasks_to_poll.is_empty() {
            for id in self.tasks_to_poll.drain() {
                if let Entry::Occupied(mut entry) = self.tasks.entry(id) {
                    if entry.get_mut().poll().is_ready() {
                        entry.remove();
                    }
                }
            }

            self.sync();
        }

        if self.tasks.is_empty() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl ExecutorProxy {
    pub fn spawn<F: Future<Output = ()> + 'static>(&mut self, future: F) -> TaskId {
        let id = self.task_counter.get_and_inc();
        let future = Box::pin(future);
        self.new_tasks.push((id, future));
        log::trace!("task queued: {id:?}");
        id
    }
}
