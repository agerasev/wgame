use std::{cell::Cell, rc::Rc};

use anyhow::{Context, Result};
use wgame_common::{Frame as CommonFrame, Window};

use crate::frame::Frame;

pub struct Surface<'a> {
    pub(crate) inner: wgpu::Surface<'a>,
    adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    size: Rc<Cell<(u32, u32)>>,
    pub(crate) format: wgpu::TextureFormat,
}

impl<'a> Surface<'a> {
    pub async fn new<W: Window<Inner: Into<wgpu::SurfaceTarget<'a>>>>(window: &W) -> Result<Self> {
        let size = window.size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());

        let surface = instance
            .create_surface(window.inner())
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
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await
            .context("Failed to create device")?;

        let caps = surface.get_capabilities(&adapter);

        let this = Self {
            inner: surface,
            adapter,
            device,
            queue,
            size: Rc::new(Cell::new(size)),
            format: caps.formats[0],
        };

        // Configure surface for the first time
        this.configure();

        Ok(this)
    }

    fn configure(&self) {
        let size = self.size.get();
        let surface_config = self
            .inner
            .get_default_config(&self.adapter, size.0, size.1)
            .unwrap();
        self.inner.configure(&self.device, &surface_config);
    }

    fn resize(&self, new_size: (u32, u32)) {
        self.size.set(new_size);

        // Reconfigure the surface
        self.configure();
    }

    pub fn create_frame<F: CommonFrame>(&self, common: F) -> Result<Frame<'a, '_, F>> {
        if let Some(new_size) = common.resized() {
            self.resize(new_size);
        }
        Frame::new(self, common)
    }
}
