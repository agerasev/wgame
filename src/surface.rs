use std::error::Error;

use winit::{dpi::PhysicalSize, window::Window};

pub trait SurfaceBuilder {
    type Surface: Surface + 'static;
    type Error: Error + 'static;
    fn build(&self, window: Window) -> Result<Self::Surface, Self::Error>;
}

pub trait Surface {
    fn window(&self) -> &Window;
    fn window_mut(&mut self) -> &mut Window;

    fn resize(&mut self, size: PhysicalSize<u32>);
}
