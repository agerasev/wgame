#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

mod frame;
mod object;
pub mod registry;
mod renderer;
mod texture;
pub mod types;

pub use self::{
    frame::Frame,
    object::{BytesSink, Model, Object, ObjectExt, Transformed, Vertices},
    registry::Registry,
    texture::Texture,
};

pub use wgpu;

use alloc::rc::Rc;
use core::cell::Cell;

use anyhow::{Context, Result};

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

#[derive(Clone)]
pub struct State<'a>(Rc<InnerState<'a>>);

struct InnerState<'a> {
    config: Config,
    surface: wgpu::Surface<'a>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: wgpu::TextureFormat,
    size: Cell<(u32, u32)>,
    registry: Registry,
}

impl<'a> InnerState<'a> {
    async fn new(
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
            })
            .await
            .context("Failed to create device")?;

        let caps = surface.get_capabilities(&adapter);

        let registry = Registry::new(&device);

        let this = Self {
            config,
            surface,
            adapter,
            device,
            queue,
            format: caps.formats[0],
            size: Default::default(),
            registry,
        };

        Ok(this)
    }

    fn configure(&self) {
        let size = self.size.get();
        if let (0, _) | (_, 0) = size {
            log::debug!("Invalid surface size: {size:?}, skipping configuration");
            return;
        }
        let surface_config = self
            .surface
            .get_default_config(&self.adapter, size.0, size.1)
            .unwrap();
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                present_mode: self.config.present_mode,
                ..surface_config
            },
        );
    }
}

impl<'a> State<'a> {
    pub async fn new(
        config: Config,
        window_handle: impl Into<wgpu::SurfaceTarget<'a>>,
    ) -> Result<Self> {
        Ok(State(Rc::new(
            InnerState::new(config, window_handle).await?,
        )))
    }

    pub fn size(&self) -> (u32, u32) {
        self.0.size.get()
    }
    pub fn resize(&self, new_size: (u32, u32)) {
        self.0.size.set(new_size);
        self.0.configure();
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.0.device
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.0.queue
    }
    pub fn format(&self) -> wgpu::TextureFormat {
        self.0.format
    }

    pub fn frame(&mut self) -> Result<Frame<'a>> {
        Frame::new(self.clone())
    }

    pub fn registry(&self) -> &Registry {
        &self.0.registry
    }
}
