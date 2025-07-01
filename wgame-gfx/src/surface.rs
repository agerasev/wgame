use std::{cell::Cell, rc::Rc};

use anyhow::{Context, Result};
use wgame_common::{Frame as CommonFrame, Window as CommonWindow};

use crate::frame::Frame;

pub(crate) struct State<'a> {
    pub surface: wgpu::Surface<'a>,
    adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub format: wgpu::TextureFormat,
    size: Rc<Cell<(u32, u32)>>,
}

impl<'a> State<'a> {
    async fn new(window_handle: impl Into<wgpu::SurfaceTarget<'a>>) -> Result<Self> {
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
        let size = self.size.get();
        let surface_config = self
            .surface
            .get_default_config(&self.adapter, size.0, size.1)
            .unwrap();
        self.surface.configure(&self.device, &surface_config);
    }

    fn resize(&self, new_size: (u32, u32)) {
        self.size.set(new_size);
        self.configure();
    }

    pub fn create_frame<F: CommonFrame>(&self, common: F) -> Result<Frame<'a, '_, F>> {
        if let Some(new_size) = common.resized() {
            self.resize(new_size);
        }
        Frame::new(self, common)
    }
}

pub struct Surface<'a, W: CommonWindow> {
    window: W,
    state: Rc<State<'a>>,
}

impl<'a, 'b, W: CommonWindow> Surface<'a, W> {
    pub async fn new(window: W) -> Result<Self>
    where
        W::Handle: Into<wgpu::SurfaceTarget<'a>>,
    {
        let state = State::new(window.handle()).await?;
        state.resize(window.size());
        Ok(Self {
            window,
            state: Rc::new(state),
        })
    }

    pub(crate) fn state(&self) -> &Rc<State<'a>> {
        &self.state
    }

    pub async fn next_frame(&mut self) -> Result<Option<Frame<'a, '_, W::Frame<'_>>>> {
        match self.window.next_frame().await {
            None => Ok(None),
            Some(common) => self.state.create_frame(common).map(Some),
        }
    }
}
