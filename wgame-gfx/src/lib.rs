#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

mod frame;
pub mod library;
mod object;
pub mod types;

pub use frame::Frame;
pub use library::Library;
pub use object::{Object, ObjectExt, Transformed};

use alloc::rc::Rc;
use core::cell::Cell;

use anyhow::{Context, Result};

pub type SharedState<'a> = Rc<State<'a>>;

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: wgpu::TextureFormat,
    size: Rc<Cell<(u32, u32)>>,
}

impl<'a> State<'a> {
    pub async fn new(window_handle: impl Into<wgpu::SurfaceTarget<'a>>) -> Result<Self> {
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
                required_features: wgpu::Features::FLOAT32_FILTERABLE | wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await
            .context("Failed to create device")?;

        let caps = surface.get_capabilities(&adapter);

        let this = Self {
            surface,
            adapter,
            device,
            queue,
            size: Default::default(),
            format: caps.formats[0],
        };

        Ok(this)
    }

    pub fn size(&self) -> (u32, u32) {
        self.size.get()
    }

    fn configure(&self) {
        let size = self.size();
        if let (0, _) | (_, 0) = size {
            log::warn!("Invalid surface size: {size:?}, skipping configuration");
            return;
        }
        let surface_config = self
            .surface
            .get_default_config(&self.adapter, size.0, size.1)
            .unwrap();
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                present_mode: wgpu::PresentMode::AutoVsync,
                ..surface_config
            },
        );
    }

    pub fn resize(&self, new_size: (u32, u32)) {
        self.size.set(new_size);
        self.configure();
    }

    pub fn frame(self: &Rc<Self>) -> Result<Frame<'a>> {
        Frame::new(self.clone())
    }
}
