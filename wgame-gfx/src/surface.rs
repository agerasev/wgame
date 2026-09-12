use anyhow::{Context as _, Result};

use crate::{Config, Frame, Graphics};

pub struct Surface<'a> {
    config: Config,
    surface: wgpu::Surface<'a>,
    state: Graphics,
    size: (u32, u32),
    pending: Option<wgpu::SurfaceTexture>,
}

impl<'a> Surface<'a> {
    pub async fn new(
        config: Config,
        window_handle: impl Into<wgpu::SurfaceTarget<'a>>,
    ) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());

        let surface = instance
            .create_surface(window_handle)
            .context("Failed to create surface")?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .context("Failed to find an appropriate adapter")?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
                experimental_features: Default::default(),
            })
            .await
            .context("Failed to create device")?;

        let caps = surface.get_capabilities(&adapter);
        let format = (caps.formats.iter().copied())
            .find(|format| !format.is_srgb())
            .unwrap_or_else(|| caps.formats[0]);

        let this = Self {
            config,
            surface,
            state: Graphics::new(adapter, device, queue, format),
            size: Default::default(),
            pending: None,
        };

        Ok(this)
    }

    fn configure(&self) {
        let size = self.size;
        if let (0, _) | (_, 0) = size {
            log::debug!("Invalid surface size: {size:?}, skipping configuration");
            return;
        }
        let surface_config = self
            .surface
            .get_default_config(self.state.adapter(), size.0, size.1)
            .unwrap();
        self.surface.configure(
            self.state.device(),
            &wgpu::SurfaceConfiguration {
                format: self.state.format(),
                present_mode: self.config.present_mode,
                ..surface_config
            },
        );
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }
    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.pending = None;
        self.size = new_size;
        self.configure();
    }

    /// Acquire a frame without borrowing the surface for the frame's lifetime.
    /// Returns false for zero size or a transient acquisition failure; request
    /// another OS redraw before retrying. Consume a successful acquisition with `frame`.
    pub fn prepare_frame(&mut self) -> Result<bool> {
        if self.pending.is_some() {
            return Ok(true);
        }
        if self.size.0 == 0 || self.size.1 == 0 {
            return Ok(false);
        }
        match self.surface.get_current_texture() {
            Ok(texture) => {
                self.pending = Some(texture);
                Ok(true)
            }
            Err(error) => match surface_action(&error) {
                SurfaceAction::Retry => Ok(false),
                SurfaceAction::Reconfigure => {
                    self.configure();
                    Ok(false)
                }
                SurfaceAction::Fail => Err(error.into()),
            },
        }
    }
    pub(crate) fn take_texture(&mut self) -> Result<wgpu::SurfaceTexture> {
        if let Some(texture) = self.pending.take() {
            Ok(texture)
        } else {
            Ok(self.surface.get_current_texture()?)
        }
    }

    pub fn frame(&mut self) -> Result<Frame<'a, '_>> {
        Frame::new(self)
    }

    pub fn state(&self) -> &Graphics {
        &self.state
    }
}

#[derive(Debug, PartialEq)]
enum SurfaceAction {
    Retry,
    Reconfigure,
    Fail,
}
fn surface_action(error: &wgpu::SurfaceError) -> SurfaceAction {
    match error {
        wgpu::SurfaceError::Timeout => SurfaceAction::Retry,
        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => SurfaceAction::Reconfigure,
        _ => SurfaceAction::Fail,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transient_errors_are_recoverable() {
        assert_eq!(
            surface_action(&wgpu::SurfaceError::Timeout),
            SurfaceAction::Retry
        );
        assert_eq!(
            surface_action(&wgpu::SurfaceError::Lost),
            SurfaceAction::Reconfigure
        );
        assert_eq!(
            surface_action(&wgpu::SurfaceError::Outdated),
            SurfaceAction::Reconfigure
        );
        assert_eq!(
            surface_action(&wgpu::SurfaceError::OutOfMemory),
            SurfaceAction::Fail
        );
        assert_eq!(
            surface_action(&wgpu::SurfaceError::Other),
            SurfaceAction::Fail
        );
    }
}
