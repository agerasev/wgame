use alloc::{boxed::Box, rc::Rc, sync::Arc, vec::Vec};
use core::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use futures::task::{ArcWake, waker};
use hashbrown::{
    HashSet,
    hash_map::{Entry, HashMap},
};
use winit::event_loop::EventLoopProxy;

type FutureObj = Pin<Box<dyn Future<Output = ()>>>;

use crate::{app::UserEvent, output::DefaultFallible};

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
    output: Rc<dyn DefaultFallible>,
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
        proxy: Rc<dyn DefaultFallible>,
    ) -> Self {
        let data = Arc::new(TaskData { id, event_loop });
        Self {
            future,
            waker: waker(data),
            output: proxy,
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
    new_tasks: Vec<(TaskId, FutureObj, Rc<dyn DefaultFallible>)>,
    tasks_to_terminate: HashSet<TaskId>,
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

        for (id, future, output) in proxy.new_tasks.drain(..) {
            let task = Task::new(id, self.event_loop.clone(), future, output);
            assert!(self.tasks.insert(id, task).is_none());
            self.tasks_to_poll.insert(id);
            log::trace!("task spawned: {id:?}");
        }

        for id in proxy.tasks_to_terminate.drain() {
            if self.tasks.remove(&id).is_some() {
                log::trace!("task terminated: {id:?}");
            } else {
                log::error!("task not found: {id:?}");
            }
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
                if self.proxy.borrow().tasks_to_terminate.contains(&id) {
                    // Don't poll tasks that will be terminated
                    continue;
                }
                if let Entry::Occupied(mut entry) = self.tasks.entry(id) {
                    if entry.get_mut().poll().is_ready() {
                        entry.remove();
                    }
                } else {
                    log::error!("Task {id:?} registered to poll, but not found");
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

    pub fn terminate_task(&mut self, task_id: TaskId) {
        self.tasks_to_poll.retain(|id| *id != task_id);
        self.tasks.retain(|id, task| {
            if *id != task_id {
                true
            } else {
                task.output.set_failed();
                false
            }
        });
    }
}

impl ExecutorProxy {
    pub fn spawn<F: Future<Output = ()> + 'static>(
        &mut self,
        future: F,
        output: Rc<dyn DefaultFallible>,
    ) -> TaskId {
        let id = self.task_counter.get_and_inc();
        let future = Box::pin(future);
        self.new_tasks.push((id, future, output));
        id
    }

    pub fn terminate(&mut self, id: TaskId) {
        self.tasks_to_terminate.insert(id);
    }
}
