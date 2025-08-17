#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

pub use wgame_app as app;
pub use wgame_gfx as gfx;
pub use wgame_macros::{app, window};

#[cfg(feature = "fs")]
pub use wgame_fs as fs;
#[cfg(feature = "img")]
pub use wgame_img as img;
#[cfg(feature = "shapes")]
pub use wgame_shapes as shapes;
#[cfg(feature = "text")]
pub use wgame_text as text;
#[cfg(feature = "utils")]
pub use wgame_utils as utils;

pub use anyhow::{Error, Result};
pub use app::runtime::{sleep, spawn};

use core::ops::{Deref, DerefMut};

use wgame_app::{
    runtime::{TaskHandle, WindowError},
    window::Suspended,
};

#[macro_export]
macro_rules! run_app {
    ($main:ident, $app_fn:expr) => {
        $crate::app::entry!($crate::app, $main, $app_fn);
    };
}

#[macro_export]
macro_rules! run_window {
    ($main:ident, $window_fn:expr) => {
        $crate::run_app!($main, $crate::open_window!($window_fn));
    };
}

#[macro_export]
macro_rules! open_window {
    ($window_fn:expr) => {{
        use $crate::{WindowConfig, within_window};

        async || {
            within_window(WindowConfig::default(), async |window| {
                log::info!("Window opened");
                let result = $window_fn(window).await;
                log::info!("Window closed");
                result
            })
            .await
        }
    }};
}

#[derive(Clone, Default, Debug)]
pub struct WindowConfig {
    pub app: app::WindowAttributes,
    pub gfx: gfx::Config,
}

#[derive(Clone)]
pub struct Runtime(pub app::Runtime);

impl Runtime {
    pub fn current() -> Self {
        Runtime(app::Runtime::current())
    }

    pub async fn create_windowed_task<T, F>(
        &self,
        config: WindowConfig,
        window_main: F,
    ) -> Result<TaskHandle<Result<Result<T>, Suspended>>>
    where
        T: 'static,
        F: AsyncFnOnce(Window<'_>) -> Result<T> + 'static,
    {
        Ok(self
            .0
            .create_windowed_task(config.app, async move |app_window| {
                let window = Window::new(app_window, config.gfx).await?;
                window_main(window).await
            })
            .await?)
    }
}

pub async fn within_window<T, F>(
    config: WindowConfig,
    window_main: F,
) -> Result<T, WindowError<Error>>
where
    T: 'static,
    F: AsyncFnOnce(Window<'_>) -> Result<T> + 'static,
{
    let result = Runtime::current()
        .create_windowed_task(config, window_main)
        .await?
        .await;
    match result {
        Ok(r) => match r {
            Ok(x) => Ok(x),
            Err(e) => Err(WindowError::Other(e)),
        },
        Err(Suspended) => Err(WindowError::Suspended),
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

impl Frame<'_, '_> {
    pub fn resized(&self) -> Option<(u32, u32)> {
        self.app.resized()
    }
}
