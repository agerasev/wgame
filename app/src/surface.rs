use std::{any::Any, error::Error, sync::Arc};

use winit::{dpi::PhysicalSize, window::Window};

pub trait SurfaceBuilder {
    type Surface: Surface + 'static;
    type Error: Error;
    fn build(&self, window: &Arc<Window>) -> Result<Self::Surface, Self::Error>;
}

pub trait Surface: Any {
    fn resize(&mut self, size: PhysicalSize<u32>);
}

pub trait Renderer<S: Surface> {
    type Output;
    type Error: Error;
    fn render(self, surface: &mut S) -> Result<Self::Output, Self::Error>;
}

impl<S, E, F> SurfaceBuilder for F
where
    S: Surface + 'static,
    E: Error + 'static,
    F: Fn(&Arc<Window>) -> Result<S, E>,
{
    type Surface = S;
    type Error = E;
    fn build(&self, window: &Arc<Window>) -> Result<Self::Surface, Self::Error> {
        (self)(window)
    }
}

pub struct DummySurface;

impl Surface for DummySurface {
    fn resize(&mut self, size: PhysicalSize<u32>) {
        log::info!("Resizing dummy surface to {size:?}")
    }
}

impl<S, T, E, F> Renderer<S> for F
where
    S: Surface,
    E: Error,
    F: FnOnce(&mut S) -> Result<T, E>,
{
    type Output = T;
    type Error = E;
    fn render(self, surface: &mut S) -> Result<Self::Output, Self::Error> {
        (self)(surface)
    }
}
