use core::ops::{Deref, DerefMut};

use anyhow::Result;

use crate::{
    app::{self, Runtime},
    config::WindowConfig,
    gfx,
};

pub use app::{WindowError, WindowedTask};

pub fn create_windowed_task<T, F>(
    rt: &Runtime,
    config: WindowConfig,
    window_fn: F,
) -> WindowedTask<Result<T, gfx::Error>>
where
    T: 'static,
    F: AsyncFnOnce(Window<'_>) -> T + 'static,
{
    app::create_windowed_task(rt, config.app, async move |app_window| {
        let window = Window::new(app_window, config.gfx).await?;
        Ok(window_fn(window).await)
    })
}

pub async fn within_window<T, F>(config: WindowConfig, window_fn: F) -> Result<T>
where
    T: 'static,
    F: AsyncFnOnce(Window<'_>) -> Result<T> + 'static,
{
    create_windowed_task(&Runtime::current(), config, window_fn).await??
}

pub struct Window<'a> {
    gfx: gfx::Surface<'a>,
    app: app::Window<'a>,
}

impl<'a> Window<'a> {
    async fn new(app: app::Window<'a>, gfx_cfg: gfx::Config) -> Result<Self> {
        let mut gfx = gfx::Surface::new(gfx_cfg, app.raw()).await?;
        gfx.resize(app.size());
        Ok(Self { app, gfx })
    }

    pub async fn next_frame(&mut self) -> Result<Option<Frame<'a, '_>>> {
        match self.app.request_redraw().await {
            None => Ok(None),
            Some(redraw) => {
                if let Some(size) = redraw.resized() {
                    self.gfx.resize(size);
                }
                Ok(Some(Frame {
                    app: redraw,
                    gfx: Some(self.gfx.frame()?),
                }))
            }
        }
    }

    pub fn graphics(&self) -> &gfx::Graphics {
        self.gfx.state()
    }
}

pub struct Frame<'a, 'b> {
    gfx: Option<gfx::Frame<'a, 'b>>,
    app: app::window::Redraw<'b>,
}

impl<'a, 'b> Deref for Frame<'a, 'b> {
    type Target = gfx::Frame<'a, 'b>;
    fn deref(&self) -> &Self::Target {
        self.gfx.as_ref().unwrap()
    }
}

impl DerefMut for Frame<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.gfx.as_mut().unwrap()
    }
}

impl Drop for Frame<'_, '_> {
    fn drop(&mut self) {
        self.app.pre_present();
        self.gfx.take().unwrap().present();
    }
}

impl Frame<'_, '_> {
    pub fn resized(&self) -> Option<(u32, u32)> {
        self.app.resized()
    }
}
