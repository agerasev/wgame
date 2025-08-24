use crate::{
    app::{Size, WindowAttributes},
    gfx::{self, PresentMode},
};

#[derive(Clone, Default, Debug)]
pub struct WindowConfig {
    pub app: WindowAttributes,
    pub gfx: gfx::Config,
}

impl WindowConfig {
    pub fn title(self, title: &str) -> Self {
        Self {
            app: self.app.with_title(title),
            ..self
        }
    }
    pub fn size(self, size: (u32, u32)) -> Self {
        Self {
            app: self.app.with_inner_size(Size::new(size.0, size.1)),
            ..self
        }
    }
    pub fn resizable(self, resizable: bool) -> Self {
        Self {
            app: self.app.with_resizable(resizable),
            ..self
        }
    }

    pub fn vsync(self, vsync: bool) -> Self {
        Self {
            gfx: gfx::Config {
                present_mode: if vsync {
                    PresentMode::AutoVsync
                } else {
                    PresentMode::AutoNoVsync
                },
            },
            ..self
        }
    }
}
