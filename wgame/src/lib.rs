#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

pub use wgame_app as app;
pub use wgame_gfx as gfx;
pub use wgame_macros::main;

pub use anyhow::{Error, Result};
pub use app::{runtime::JoinHandle, timer::Timer};

use alloc::rc::Rc;
use core::ops::Deref;

#[macro_export]
macro_rules! run {
    ($main:ident, $async_main:path) => {
        async fn __wgame_app_wrapper(app_rt: $crate::app::Runtime) {
            $async_main($crate::Runtime(app_rt)).await
        }
        $crate::app::entry!($crate::app, $main, __wgame_app_wrapper);
    };
}

/// TODO: Import from `wgame_gfx`.
type GraphicsConfig = ();

#[derive(Clone, Default, Debug)]
pub struct WindowConfig {
    pub app: app::WindowAttributes,
    pub gfx: GraphicsConfig,
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
    gfx: gfx::SharedState<'a>,
    app: app::Window<'a>,
}

impl<'a> Window<'a> {
    async fn new(app: app::Window<'a>, _gfx_cfg: GraphicsConfig) -> Result<Self> {
        let gfx = Rc::new(gfx::State::new(app.handle()).await?);
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

    pub fn graphics(&self) -> &Rc<gfx::State<'a>> {
        &self.gfx
    }
}

pub struct Frame<'a, 'b> {
    gfx: Option<gfx::Frame<'a>>,
    app: app::window::Redraw<'b>,
}

impl<'a> Deref for Frame<'a, '_> {
    type Target = gfx::Frame<'a>;
    fn deref(&self) -> &Self::Target {
        self.gfx.as_ref().unwrap()
    }
}

impl Drop for Frame<'_, '_> {
    fn drop(&mut self) {
        self.app.pre_present();
        self.gfx.take().unwrap().present();
    }
}
