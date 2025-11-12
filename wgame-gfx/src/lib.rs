#![forbid(unsafe_code)]

mod collector;
mod camera;
mod frame;
mod instance;
pub mod modifiers;
mod object;
pub mod renderer;
mod resource;
mod state;
pub mod types;
pub mod utils;

pub use self::{
    collector::Collector,
    camera::Camera,
    frame::Frame,
    instance::{Instance, InstanceExt},
    object::{Object, ObjectExt},
    renderer::Renderer,
    resource::Resource,
    state::Graphics,
};
pub use anyhow::Error;
pub use wgpu::PresentMode;

use anyhow::{Context as _, Result};

#[derive(Clone, Debug)]
pub struct Config {
    pub present_mode: wgpu::PresentMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            present_mode: wgpu::PresentMode::AutoVsync,
        }
    }
}

pub struct Surface<'a> {
    config: Config,
    surface: wgpu::Surface<'a>,
    state: Graphics,
    size: (u32, u32),
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
        let format = caps.formats[0];

        let this = Self {
            config,
            surface,
            state: Graphics {
                adapter,
                device,
                queue,
                format,
            },
            size: Default::default(),
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
            .get_default_config(&self.state.adapter, size.0, size.1)
            .unwrap();
        self.surface.configure(
            self.state.device(),
            &wgpu::SurfaceConfiguration {
                present_mode: self.config.present_mode,
                ..surface_config
            },
        );
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }
    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.size = new_size;
        self.configure();
    }

    pub fn frame(&mut self) -> Result<Frame<'a, '_>> {
        Frame::new(self)
    }

    pub fn state(&self) -> &Graphics {
        &self.state
    }
}
