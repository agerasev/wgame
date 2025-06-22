use std::{
    any::Any,
    cell::{RefCell, RefMut},
    error::Error,
    mem,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use futures::{Stream, future::FusedFuture};
use pin_project::pin_project;
use thiserror::Error;
use winit::{
    error::OsError, event::WindowEvent, event_loop::ActiveEventLoop, window::WindowAttributes,
};

use crate::{
    app::{ActualWindowState, AppProxy, WindowState},
    surface::{Renderer, SurfaceBuilder},
};

#[derive(Debug, Error)]
pub enum CreateError<S: Error> {
    Window(#[from] OsError),
    Surface(S),
}

#[derive(Debug, Error)]
pub enum RenderError<S: Error, R: Error> {
    Resume(#[from] CreateError<S>),
    Render(R),
}

pub struct Window<S: SurfaceBuilder> {
    app: AppProxy,
    state: Rc<RefCell<WindowState>>,
    builder: S,
}

impl<S: SurfaceBuilder> Drop for Window<S> {
    fn drop(&mut self) {
        self.app.state.borrow_mut().remove_window(&self.state);
    }
}

impl<S: SurfaceBuilder> Window<S> {
    pub(crate) fn new(app: AppProxy, attributes: WindowAttributes, builder: S) -> Self {
        let state = app.state.borrow_mut().new_window_state(attributes);
        Self {
            app,
            state,
            builder,
        }
    }

    pub(crate) fn create(&mut self) -> Create<'_, S> {
        Create {
            owner: Some(self),
            state: Rc::new(RefCell::new(CreateState { error: None })),
        }
    }

    pub fn render<R: Renderer<S::Surface>>(&mut self, renderer: R) -> Render<'_, S, R> {
        Render {
            owner: Some(self),
            state: Rc::new(RefCell::new(CreateState { error: None })),
            renderer: Some(renderer),
        }
    }

    pub fn events(&mut self) -> EventPipe<'_> {
        EventPipe { state: &self.state }
    }

    pub fn is_closed(&self) -> bool {
        self.state.borrow().close_requested
    }
    pub fn closed(&mut self) -> Closed<'_> {
        Closed { state: &self.state }
    }

    fn poll_create(
        &mut self,
        create: &Rc<RefCell<CreateState>>,
    ) -> Poll<Result<RefMut<'_, ActualWindowState>, CreateError<S::Error>>> {
        // Try get actual window state
        match RefMut::filter_map(self.state.borrow_mut(), |s| s.actual.as_mut()).ok() {
            None => {
                // Check if an error occured
                if let Some(err) = create.borrow_mut().error.take() {
                    return Poll::Ready(Err(CreateError::Window(err)));
                }

                // Check app is not suspended
                if !self.app.state.borrow().is_active() {
                    return Poll::Pending;
                }

                // Create actual window state
                let app = self.app.state.clone();
                let state = self.state.clone();
                let create = create.clone();
                self.app.executor.borrow_mut().add_loop_call(
                    move |event_loop: &ActiveEventLoop| {
                        if let Err(err) = app.borrow_mut().create_actual_window(&state, event_loop)
                        {
                            create.borrow_mut().error = Some(err);
                        }
                        if let Some(waker) = state.borrow_mut().waker.take() {
                            waker.wake();
                        }
                    },
                );
                Poll::Pending
            }

            Some(mut actual) => {
                if actual.surface.is_none() {
                    match self.builder.build(&actual.window) {
                        Ok(surface) => {
                            actual.surface = Some(Box::new(surface));
                        }
                        Err(err) => {
                            return Poll::Ready(Err(CreateError::Surface(err)));
                        }
                    }
                }
                Poll::Ready(Ok(actual))
            }
        }
    }

    #[allow(clippy::type_complexity)]
    fn poll_render<R: Renderer<S::Surface>>(
        &mut self,
        create: &Rc<RefCell<CreateState>>,
        renderer: &mut Option<R>,
    ) -> Poll<Result<Option<R::Output>, RenderError<S::Error, R::Error>>> {
        // If window closed
        if self.state.borrow().close_requested {
            return Poll::Ready(Ok(None));
        }

        // Try create window
        let mut actual = match self.poll_create(create)? {
            Poll::Ready(actual) => actual,
            Poll::Pending => return Poll::Pending,
        };

        // Whether redraw requested
        if !mem::replace(&mut actual.redraw_requested, false) {
            actual.window.request_redraw();
            return Poll::Pending;
        }

        // Render
        let dyn_surface = actual
            .surface
            .as_mut()
            .expect("Surface hasn't been created");
        let surface = (dyn_surface.as_mut() as &mut dyn Any)
            .downcast_mut::<S::Surface>()
            .expect("Error downcasting surface");
        Poll::Ready(
            match renderer.take().expect("Renderer is empty").render(surface) {
                Ok(out) => Ok(Some(out)),
                Err(err) => Err(RenderError::Render(err)),
            },
        )
    }
}

struct CreateState {
    error: Option<OsError>,
}

pub struct Create<'a, S: SurfaceBuilder> {
    owner: Option<&'a mut Window<S>>,
    state: Rc<RefCell<CreateState>>,
}

impl<'a, S: SurfaceBuilder> Future for Create<'a, S> {
    type Output = Result<(), CreateError<S::Error>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let owner = self
            .owner
            .take()
            .expect("FusedFuture polled again after it returned Ready");

        let poll = owner.poll_create(&self.state).map_ok(|_| ());

        if poll.is_pending() {
            owner.state.borrow_mut().waker = Some(cx.waker().clone());
            self.owner = Some(owner);
        }

        poll
    }
}

impl<S: SurfaceBuilder> FusedFuture for Create<'_, S> {
    fn is_terminated(&self) -> bool {
        self.owner.is_none()
    }
}

#[pin_project]
pub struct Render<'a, S: SurfaceBuilder, R: Renderer<S::Surface>> {
    owner: Option<&'a mut Window<S>>,
    state: Rc<RefCell<CreateState>>,
    renderer: Option<R>,
}

impl<'a, S: SurfaceBuilder, R: Renderer<S::Surface>> Future for Render<'a, S, R> {
    type Output = Result<Option<R::Output>, RenderError<S::Error, R::Error>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let owner = this
            .owner
            .take()
            .expect("FusedFuture polled again after it returned Ready");

        let poll = owner.poll_render(this.state, this.renderer);

        if poll.is_pending() {
            owner.state.borrow_mut().waker = Some(cx.waker().clone());
            *this.owner = Some(owner);
        }

        poll
    }
}

impl<S: SurfaceBuilder, R: Renderer<S::Surface>> FusedFuture for Render<'_, S, R> {
    fn is_terminated(&self) -> bool {
        self.owner.is_none()
    }
}

pub struct EventPipe<'a> {
    state: &'a RefCell<WindowState>,
}

impl Stream for EventPipe<'_> {
    type Item = WindowEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut state = self.state.borrow_mut();
        match state.events.pop_front() {
            Some(event) => Poll::Ready(Some(event)),
            None => {
                if state.close_requested {
                    Poll::Ready(None)
                } else {
                    state.waker = Some(cx.waker().clone());
                    Poll::Pending
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.state.borrow().events.len();
        (len, Some(len))
    }
}

pub struct Closed<'a> {
    state: &'a RefCell<WindowState>,
}

impl Future for Closed<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.borrow_mut();
        if state.close_requested {
            Poll::Ready(())
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl FusedFuture for Closed<'_> {
    fn is_terminated(&self) -> bool {
        self.state.borrow().close_requested
    }
}
