use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use futures::task::{ArcWake, waker};
use hashbrown::{
    HashSet,
    hash_map::{Entry, HashMap},
};
use winit::event_loop::EventLoopProxy;

use crate::app::UserEvent;

type FutureObj = Pin<Box<dyn Future<Output = ()>>>;
type Terminator = Box<dyn FnOnce()>;

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
    output: Terminator,
}

struct TaskData {
    id: TaskId,
    event_loop: EventLoopProxy<UserEvent>,
}

impl ArcWake for TaskData {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Late wakes after shutdown are harmless.
        let _ = arc_self.event_loop.send_event(UserEvent {
            task_id: arc_self.id,
        });
    }
}

impl Task {
    fn poll(&mut self) -> Poll<()> {
        let mut cx = Context::from_waker(&self.waker);
        match self.future.as_mut().poll(&mut cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(()) => Poll::Ready(()),
        }
    }
}

pub struct Executor {
    make_waker: Rc<dyn Fn(TaskId) -> Waker>,
    tasks: HashMap<TaskId, Task>,
    tasks_to_poll: HashSet<TaskId>,
    proxy: Rc<RefCell<ExecutorProxy>>,
}

#[derive(Default)]
pub struct ExecutorProxy {
    task_counter: TaskId,
    new_tasks: Vec<(TaskId, FutureObj, Terminator)>,
    tasks_to_terminate: HashSet<TaskId>,
}

impl Executor {
    pub fn new(event_loop: EventLoopProxy<UserEvent>) -> Self {
        Self::with_waker_factory(move |id| {
            waker(Arc::new(TaskData {
                id,
                event_loop: event_loop.clone(),
            }))
        })
    }

    pub(crate) fn with_waker_factory(make_waker: impl Fn(TaskId) -> Waker + 'static) -> Self {
        Self {
            make_waker: Rc::new(make_waker),
            tasks: HashMap::default(),
            tasks_to_poll: HashSet::default(),
            proxy: Rc::new(RefCell::new(ExecutorProxy::default())),
        }
    }

    pub fn proxy(&self) -> Rc<RefCell<ExecutorProxy>> {
        self.proxy.clone()
    }

    fn sync(&mut self) {
        loop {
            // Never hold the proxy borrow while dropping futures or invoking callbacks:
            // both can spawn or cancel other tasks.
            let (new_tasks, cancelled) = {
                let mut proxy = self.proxy.borrow_mut();
                (
                    std::mem::take(&mut proxy.new_tasks),
                    std::mem::take(&mut proxy.tasks_to_terminate),
                )
            };
            if new_tasks.is_empty() && cancelled.is_empty() {
                break;
            }
            for (id, future, output) in new_tasks {
                let task = Task {
                    future,
                    output,
                    waker: (self.make_waker)(id),
                };
                assert!(self.tasks.insert(id, task).is_none());
                self.tasks_to_poll.insert(id);
            }
            for id in cancelled {
                self.cancel_running(id);
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
        self.proxy.borrow_mut().terminate(task_id);
        self.sync();
    }

    fn cancel_running(&mut self, task_id: TaskId) {
        self.tasks_to_poll.remove(&task_id);
        if let Some(task) = self.tasks.remove(&task_id) {
            let Task { future, output, .. } = task;
            drop(future);
            output();
        }
    }
}

impl ExecutorProxy {
    pub fn spawn<F: Future<Output = ()> + 'static, G: FnOnce() + 'static>(
        &mut self,
        future: F,
        terminator: G,
    ) -> TaskId {
        let id = self.task_counter.get_and_inc();
        let future = Box::pin(future);
        let output = Box::new(terminator);
        self.new_tasks.push((id, future, output));
        id
    }

    pub fn terminate(&mut self, id: TaskId) {
        self.tasks_to_terminate.insert(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    fn executor() -> Executor {
        Executor::with_waker_factory(|_| Waker::noop().clone())
    }

    #[test]
    fn cancellation_before_first_poll_completes_output_once() {
        let mut ex = executor();
        let calls = Rc::new(Cell::new(0));
        let count = calls.clone();
        let id = ex
            .proxy
            .borrow_mut()
            .spawn(async { panic!("cancelled future polled") }, move || {
                count.set(count.get() + 1)
            });
        ex.proxy.borrow_mut().terminate(id);
        assert!(ex.poll().is_ready());
        assert_eq!(calls.get(), 1);
        ex.terminate_task(id);
        assert_eq!(calls.get(), 1);
    }
    #[test]
    fn cancellation_drops_future_without_borrowing_proxy() {
        struct OnDrop(Rc<RefCell<ExecutorProxy>>, Rc<Cell<bool>>);
        impl Drop for OnDrop {
            fn drop(&mut self) {
                self.0.borrow_mut().spawn(async {}, || {});
                self.1.set(true);
            }
        }
        let mut ex = executor();
        let dropped = Rc::new(Cell::new(false));
        let guard = OnDrop(ex.proxy(), dropped.clone());
        let id = ex.proxy.borrow_mut().spawn(
            async move {
                let _guard = guard;
                std::future::pending::<()>().await
            },
            || {},
        );
        assert!(ex.poll().is_pending());
        ex.proxy.borrow_mut().terminate(id);
        let _ = ex.poll();
        assert!(dropped.get());
        assert!(ex.poll().is_ready());
    }
    #[test]
    fn cancellation_callback_can_spawn_a_live_task() {
        let mut ex = executor();
        let proxy = ex.proxy();
        let id = ex
            .proxy
            .borrow_mut()
            .spawn(std::future::pending(), move || {
                proxy.borrow_mut().spawn(std::future::pending(), || {});
            });
        ex.terminate_task(id);
        assert!(
            ex.poll().is_pending(),
            "new work must prevent event-loop exit"
        );
    }
    #[test]
    fn completed_task_is_not_cancelled_again() {
        let mut ex = executor();
        let id = ex
            .proxy
            .borrow_mut()
            .spawn(async {}, || panic!("completed task terminated"));
        assert!(ex.poll().is_ready());
        ex.proxy.borrow_mut().terminate(id);
        assert!(ex.poll().is_ready());
    }
}
