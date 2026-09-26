//! Window configuration.

use crate::{
    app::{LogicalSize, Size, WindowAttributes},
    gfx::{self, PresentMode},
};

/// Configuration for a window.
///
/// Combines application-level window attributes with graphics configuration.
#[derive(Clone, Default, Debug)]
pub struct WindowConfig {
    /// Window attributes from wgame_app.
    pub app: WindowAttributes,
    /// Graphics configuration.
    pub gfx: gfx::Config,
}

impl WindowConfig {
    /// Sets the window title.
    pub fn title(self, title: &str) -> Self {
        Self {
            app: self.app.with_title(title),
            ..self
        }
    }

    /// Sets the inner window size in physical pixels.
    pub fn size(self, size: (u32, u32)) -> Self {
        Self {
            app: self.app.with_inner_size(Size::new(size.0, size.1)),
            ..self
        }
    }

    /// Sets the inner window size in logical pixels, scaled by the OS.
    ///
    /// ```
    /// let config = wgame::WindowConfig::default().logical_size((800.0, 600.0));
    /// ```
    ///
    /// Also accepted by `#[wgame::window(logical_size = (800.0, 600.0))]`.
    pub fn logical_size(self, size: (f64, f64)) -> Self {
        Self {
            app: self.app.with_inner_size(LogicalSize::new(size.0, size.1)),
            ..self
        }
    }

    /// Sets whether the window is resizable.
    pub fn resizable(self, resizable: bool) -> Self {
        Self {
            app: self.app.with_resizable(resizable),
            ..self
        }
    }

    /// Sets the optional GPU features required by the application.
    ///
    /// Unsupported features cause window graphics initialization to fail. This
    /// does not change the selected native or browser backend.
    pub fn required_features(mut self, features: wgpu::Features) -> Self {
        self.gfx.required_features = features;
        self
    }

    /// Sets device limits, including compute and storage-buffer capabilities.
    ///
    /// Defaults are WebGL2-compatible and exclude compute. Use a compute-capable
    /// backend with the ordinary WebGPU limits for a compute application:
    ///
    /// ```
    /// let config = wgame::WindowConfig::default()
    ///     .required_limits(wgpu::Limits::default())
    ///     .vsync(false);
    /// ```
    ///
    /// See [`gfx::Config::required_limits`] for texture-resolution handling.
    pub fn required_limits(mut self, limits: wgpu::Limits) -> Self {
        self.gfx.required_limits = limits;
        self
    }

    /// Request the adapter's buffer limits instead of the configured sizes.
    ///
    /// This opt-in permits buffers larger than WebGPU's baseline limits without
    /// allocating memory. It leaves compute and other requirements unchanged:
    ///
    /// ```
    /// let config = wgame::WindowConfig::default()
    ///     .required_limits(wgpu::Limits::default())
    ///     .use_adapter_buffer_limits(true);
    /// ```
    ///
    /// See [`gfx::Config::use_adapter_buffer_limits`] for allocation constraints.
    pub fn use_adapter_buffer_limits(mut self, enabled: bool) -> Self {
        self.gfx.use_adapter_buffer_limits = enabled;
        self
    }

    /// Sets whether vsync is enabled, preserving GPU device requirements.
    pub fn vsync(self, vsync: bool) -> Self {
        Self {
            gfx: gfx::Config {
                present_mode: if vsync {
                    PresentMode::AutoVsync
                } else {
                    PresentMode::AutoNoVsync
                },
                ..self.gfx
            },
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WindowConfig;

    #[test]
    fn graphics_requirements_survive_window_builders() {
        let limits = wgpu::Limits {
            max_compute_workgroups_per_dimension: 128,
            ..wgpu::Limits::default()
        };
        let features = wgpu::Features::TIMESTAMP_QUERY;
        assert!(!WindowConfig::default().gfx.use_adapter_buffer_limits);
        let config = WindowConfig::default()
            .required_limits(limits.clone())
            .required_features(features)
            .use_adapter_buffer_limits(true)
            .vsync(false)
            .title("compute")
            .size((640, 480))
            .resizable(false);
        assert_eq!(config.gfx.required_limits, limits);
        assert_eq!(config.gfx.required_features, features);
        assert!(config.gfx.use_adapter_buffer_limits);
        assert_eq!(config.gfx.present_mode, wgpu::PresentMode::AutoNoVsync);
        let config = config.vsync(true);
        assert_eq!(config.gfx.required_limits, limits);
        assert_eq!(config.gfx.required_features, features);
        assert!(config.gfx.use_adapter_buffer_limits);
        assert_eq!(config.gfx.present_mode, wgpu::PresentMode::AutoVsync);
        assert!(
            !config
                .use_adapter_buffer_limits(false)
                .gfx
                .use_adapter_buffer_limits
        );
    }
}
