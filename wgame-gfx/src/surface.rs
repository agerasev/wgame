use anyhow::{Context, Result};

pub struct Surface<'a> {
    pub(crate) inner: wgpu::Surface<'a>,
    adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    size: (u32, u32),
    pub(crate) format: wgpu::TextureFormat,
}

impl<'a> Surface<'a> {
    pub async fn new<W: Into<wgpu::SurfaceTarget<'a>>>(
        window: W,
        size: (u32, u32),
    ) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());

        let surface = instance
            .create_surface(window)
            .context("Failed to create surface")?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .context("Failed to find an appropriate adapter")?;

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
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
            size,
            format: caps.formats[0],
        };

        // Configure surface for the first time
        this.configure();

        Ok(this)
    }

    fn configure(&self) {
        let surface_config = self
            .inner
            .get_default_config(&self.adapter, self.size.0, self.size.1)
            .unwrap();
        self.inner.configure(&self.device, &surface_config);
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.size = new_size;

        // Reconfigure the surface
        self.configure();
    }
}
