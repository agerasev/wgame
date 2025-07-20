#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

pub use wgame_app as app;
pub use wgame_gfx as gfx;
pub use wgame_macros::main;

#[cfg(feature = "fs")]
pub use wgame_fs as fs;
#[cfg(feature = "img")]
pub use wgame_img as img;
#[cfg(feature = "shapes")]
pub use wgame_shapes as shapes;
#[cfg(feature = "utils")]
pub use wgame_utils as utils;

pub use anyhow::{Error, Result};
pub use app::{runtime::JoinHandle, timer::Timer};

use core::ops::{Deref, DerefMut};

#[macro_export]
macro_rules! run {
    ($main:ident, $async_main:path) => {
        async fn __wgame_app_wrapper(app_rt: $crate::app::Runtime) {
            $async_main($crate::Runtime(app_rt)).await
        }
        $crate::app::entry!($crate::app, $main, __wgame_app_wrapper);
    };
}

#[derive(Clone, Default, Debug)]
pub struct WindowConfig {
    pub app: app::WindowAttributes,
    pub gfx: gfx::Config,
}

#[derive(Clone)]
pub struct Runtime(pub app::Runtime);

impl Runtime {
    pub async fn create_window<T, F>(
        &self,
        config: WindowConfig,
        window_main: F,
    ) -> Result<JoinHandle<Result<T>>>
    where
        T: 'static,
        F: AsyncFnOnce(Window<'_>) -> Result<T> + 'static,
    {
        self.0
            .create_windowed_task(config.app, async move |app_window| {
                window_main(Window::new(app_window, config.gfx).await?).await
            })
            .await
            .map_err(Error::from)
    }
}

pub struct Window<'a> {
    gfx: gfx::Surface<'a>,
    app: app::Window<'a>,
}

impl<'a> Window<'a> {
    async fn new(app: app::Window<'a>, gfx_cfg: gfx::Config) -> Result<Self> {
        let mut gfx = gfx::Surface::new(gfx_cfg, app.handle()).await?;
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
