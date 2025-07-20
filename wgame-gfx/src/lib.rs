#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

mod context;
mod frame;
mod modifiers;
mod queue;
mod registry;
mod renderer;
mod texture;
mod types;

pub use self::{
    context::{Context, ContextExt},
    frame::Frame,
    modifiers::Transformed,
    registry::Registry,
    renderer::{Instance, InstanceExt, Renderer},
    texture::Texture,
    types::*,
};
pub use wgpu::PresentMode;

use alloc::rc::Rc;

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

#[derive(Clone)]
pub struct Graphics(Rc<State>);

struct State {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: wgpu::TextureFormat,
    registry: Registry,
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
            })
            .await
            .context("Failed to create device")?;

        let caps = surface.get_capabilities(&adapter);

        let registry = Registry::new(&device);

        let this = Self {
            config,
            surface,
            state: Graphics(Rc::new(State {
                adapter,
                device,
                queue,
                format: caps.formats[0],
                registry,
            })),
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
            .get_default_config(&self.state.0.adapter, size.0, size.1)
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

impl Graphics {
    pub fn device(&self) -> &wgpu::Device {
        &self.0.device
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.0.queue
    }
    pub fn format(&self) -> wgpu::TextureFormat {
        self.0.format
    }

    pub fn registry(&self) -> &Registry {
        &self.0.registry
    }
}
