use std::{sync::Arc, time::Duration};

use wgame::{Runtime, WindowAttributes};

struct WgpuSurface<'a> {
    window: &'a winit::window::Window,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'a>,
    surface_format: wgpu::TextureFormat,
}

impl<'a> WgpuSurface<'a> {
    fn new(
        window: &'a winit::window::Window,
        instance: wgpu::Instance,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Result<Self, wgpu::CreateSurfaceError> {
        let surface = instance.create_surface(window.clone())?;
        let cap = surface.get_capabilities(&adapter);

        let this = Self {
            window: window.clone(),
            device: device.clone(),
            queue: queue.clone(),
            size: window.inner_size(),
            surface,
            surface_format: cap.formats[0],
        };

        // Configure surface for the first time
        this.configure_surface();

        Ok(this)
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;

        // reconfigure the surface
        self.configure_surface();
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Create texture view
        let surface_texture = self.surface.get_current_texture()?;
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        // Renders a GREEN screen
        let mut encoder = self.device.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                // depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // If you wanted to call any drawing commands, they would go here.

        // End the renderpass.
        drop(renderpass);

        // Submit the command in the queue to execute
        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();

        Ok(())
    }
}

#[wgame::main]
async fn main(rt: Runtime) {
    env_logger::init();
    println!("Started");

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .unwrap();
    println!("WGPU initialized");

    rt.clone()
        .create_window(WindowAttributes::default(), async move |window| {
            println!("Window created");

            let surface = WgpuSurface::new(window.raw(), instance, adapter, device, queue).unwrap();
            println!("Surface created");

            let mut counter = 0;
            while let Some(()) = window.request_redraw().await {
                surface.render().unwrap();

                println!("Rendered frame #{counter}");
                counter += 1;

                rt.sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .unwrap();
    println!("Closed");
}
