use anyhow::{Context as _, Result, bail};

use crate::{Config, Frame, Graphics};

pub struct Surface<'a> {
    config: Config,
    surface: wgpu::Surface<'a>,
    state: Graphics,
    size: (u32, u32),
    pending: Option<wgpu::SurfaceTexture>,
    depth: Option<crate::DepthBuffer>,
}

impl<'a> Surface<'a> {
    /// Create a surface without an instance display handle (Vulkan, Metal,
    /// DX12, or browser backends). For native GLES, use
    /// [`Self::with_display_handle`] so the instance connects to the same display
    /// server as the window.
    pub async fn new(
        config: Config,
        window_handle: impl Into<wgpu::SurfaceTarget<'a>>,
    ) -> Result<Self> {
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
        Self::with_instance(config, window_handle, &instance).await
    }

    /// Create a surface using the window's owned display connection.
    /// With winit, pass `ActiveEventLoop::owned_display_handle()` as `display`.
    /// The display handle must belong to the same display server as the window.
    pub async fn with_display_handle(
        config: Config,
        window_handle: impl Into<wgpu::SurfaceTarget<'a>>,
        display: impl wgpu::rwh::HasDisplayHandle + std::fmt::Debug + Send + Sync + 'static,
    ) -> Result<Self> {
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_with_display_handle_from_env(Box::new(display)),
        );
        Self::with_instance(config, window_handle, &instance).await
    }

    async fn with_instance(
        config: Config,
        window_handle: impl Into<wgpu::SurfaceTarget<'a>>,
        instance: &wgpu::Instance,
    ) -> Result<Self> {
        let surface = instance
            .create_surface(window_handle)
            .context("Failed to create surface")?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .context("Failed to find an appropriate adapter")?;
        log::debug!("Graphics adapter: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: config.required_features,
                required_limits: device_limits(&config, adapter.limits()),
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
            depth: None,
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
        if self.size != new_size {
            self.depth = (new_size.0 > 0 && new_size.1 > 0)
                .then(|| crate::DepthBuffer::new(&self.state, new_size));
        }
        self.size = new_size;
        self.configure();
    }

    /// Acquire a frame without borrowing the surface for the frame's lifetime.
    /// Returns false for zero size or a transient acquisition failure; request
    /// another OS redraw before retrying. Consume a successful acquisition with `frame`.
    /// Surface loss and validation failures return an error; a lost surface must
    /// be recreated rather than repeatedly reconfigured.
    pub fn prepare_frame(&mut self) -> Result<bool> {
        if self.pending.is_some() {
            return Ok(true);
        }
        if self.size.0 == 0 || self.size.1 == 0 {
            return Ok(false);
        }
        let result = self.surface.get_current_texture();
        match surface_action(&result) {
            SurfaceAction::Render => {
                let wgpu::CurrentSurfaceTexture::Success(texture) = result else {
                    unreachable!("only a successful acquisition is renderable");
                };
                self.pending = Some(texture);
                Ok(true)
            }
            SurfaceAction::Retry => Ok(false),
            SurfaceAction::Reconfigure => {
                // A suboptimal result owns a texture. Release it before configure,
                // which must not run while any old surface texture is alive.
                drop(result);
                self.configure();
                Ok(false)
            }
            SurfaceAction::Fail => bail!("Surface acquisition failed: {result:?}"),
        }
    }
    pub(crate) fn take_texture(&mut self) -> Result<wgpu::SurfaceTexture> {
        if !self.prepare_frame()? {
            bail!("Surface is not ready; request another redraw before acquiring a frame");
        }
        Ok(self
            .pending
            .take()
            .expect("prepare_frame must retain its texture"))
    }

    pub fn frame(&mut self) -> Result<Frame<'a, '_>> {
        Frame::new(self)
    }

    pub(crate) fn depth_view(&self) -> &wgpu::TextureView {
        self.depth
            .as_ref()
            .expect("a drawable surface has depth")
            .view()
    }

    pub fn state(&self) -> &Graphics {
        &self.state
    }
}

fn device_limits(config: &Config, adapter: wgpu::Limits) -> wgpu::Limits {
    let mut required = config.required_limits.clone();
    if config.use_adapter_buffer_limits {
        required.max_storage_buffer_binding_size = adapter.max_storage_buffer_binding_size;
        required.max_buffer_size = adapter.max_buffer_size;
    }
    required.using_resolution(adapter)
}

#[derive(Debug, PartialEq)]
enum SurfaceAction {
    Render,
    Retry,
    Reconfigure,
    Fail,
}
fn surface_action(result: &wgpu::CurrentSurfaceTexture) -> SurfaceAction {
    match result {
        wgpu::CurrentSurfaceTexture::Success(_) => SurfaceAction::Render,
        wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
            SurfaceAction::Retry
        }
        wgpu::CurrentSurfaceTexture::Suboptimal(_) | wgpu::CurrentSurfaceTexture::Outdated => {
            SurfaceAction::Reconfigure
        }
        wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Validation => {
            SurfaceAction::Fail
        }
    }
}
#[cfg(test)]
mod tests {
    use super::{SurfaceAction, device_limits, surface_action};
    use crate::Config;
    use wgpu::CurrentSurfaceTexture;

    #[test]
    fn default_buffer_limits_are_preserved() {
        let config = Config::default();
        let adapter = wgpu::Limits {
            max_storage_buffer_binding_size: 512 << 20,
            max_buffer_size: 1 << 30,
            max_texture_dimension_2d: 16384,
            ..wgpu::Limits::default()
        };
        assert!(!config.use_adapter_buffer_limits);
        assert_eq!(
            device_limits(&config, adapter.clone()),
            config.required_limits.using_resolution(adapter)
        );
    }

    #[test]
    fn adapter_buffer_limits_override_only_buffer_requirements() {
        let adapter = wgpu::Limits {
            max_storage_buffer_binding_size: 512 << 20,
            max_buffer_size: 1 << 30,
            max_texture_dimension_2d: 16384,
            ..wgpu::Limits::default()
        };
        let required_limits = wgpu::Limits {
            max_storage_buffer_binding_size: 32 << 20,
            max_buffer_size: 64 << 20,
            max_compute_workgroups_per_dimension: 128,
            max_uniform_buffer_binding_size: 16384,
            ..wgpu::Limits::default()
        };
        let config = Config {
            required_limits: required_limits.clone(),
            use_adapter_buffer_limits: true,
            ..Default::default()
        };
        let mut expected = required_limits.using_resolution(adapter.clone());
        expected.max_storage_buffer_binding_size = adapter.max_storage_buffer_binding_size;
        expected.max_buffer_size = adapter.max_buffer_size;
        assert_eq!(device_limits(&config, adapter), expected);
    }

    #[test]
    fn transient_acquisition_failures_retry_or_reconfigure() {
        for result in [
            CurrentSurfaceTexture::Timeout,
            CurrentSurfaceTexture::Occluded,
        ] {
            assert_eq!(surface_action(&result), SurfaceAction::Retry);
        }
        assert_eq!(
            surface_action(&CurrentSurfaceTexture::Outdated),
            SurfaceAction::Reconfigure
        );
    }

    #[test]
    fn lost_surfaces_and_validation_failures_do_not_retry_forever() {
        for result in [
            CurrentSurfaceTexture::Lost,
            CurrentSurfaceTexture::Validation,
        ] {
            assert_eq!(surface_action(&result), SurfaceAction::Fail);
        }
    }
}
