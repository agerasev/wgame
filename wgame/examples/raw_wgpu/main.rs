use std::{
    borrow::Cow,
    f32::consts::{FRAC_PI_3, PI},
    time::Instant,
};

use bytemuck::{Pod, Zeroable};
use wgame::{
    Runtime,
    app::{WindowAttributes, window::Frame},
};
use wgame_common::Frame as _;
use wgame_utils::FrameCounter;
use wgpu::util::DeviceExt;

struct WgpuState<'a> {
    surface: wgpu::Surface<'a>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: (u32, u32),
    format: wgpu::TextureFormat,
}

impl<'a> WgpuState<'a> {
    async fn new(window: &'a winit::window::Window) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());

        let surface = instance
            .create_surface(window)
            .expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

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
            .expect("Failed to create device");

        let caps = surface.get_capabilities(&adapter);

        let this = Self {
            surface,
            adapter,
            device,
            queue,
            size: window.inner_size().into(),
            format: caps.formats[0],
        };

        // Configure surface for the first time
        this.configure_surface();

        this
    }

    fn configure_surface(&self) {
        let surface_config = self
            .surface
            .get_default_config(&self.adapter, self.size.0, self.size.1)
            .unwrap();
        self.surface.configure(&self.device, &surface_config);
    }

    fn resize(&mut self, new_size: (u32, u32)) {
        self.size = new_size;

        // reconfigure the surface
        self.configure_surface();
    }
}

struct WgpuFrame<'a, 'b, 'c> {
    state: &'b mut WgpuState<'a>,
    inner: Frame<'c>,
    frame: Option<wgpu::SurfaceTexture>,
    view: wgpu::TextureView,
}

impl<'a> WgpuState<'a> {
    fn create_frame<'b, 'c>(&'b mut self, inner: Frame<'c>) -> WgpuFrame<'a, 'b, 'c> {
        if let Some(new_size) = inner.resized() {
            self.resize(new_size);
        }

        // Create texture view
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        WgpuFrame {
            state: self,
            inner,
            frame: Some(frame),
            view,
        }
    }
}

impl Drop for WgpuFrame<'_, '_, '_> {
    fn drop(&mut self) {
        self.inner.pre_present();
        self.frame.take().unwrap().present();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    pos: [f32; 4],
    tex_coord: [f32; 2],
}

struct TriangleScene {
    vertex_buf: wgpu::Buffer,
    uniform_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl TriangleScene {
    pub fn new(state: &WgpuState<'_>) -> Self {
        let device = &state.device;
        let swapchain_format = state.format;

        // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let vertex_data = [
            Vertex {
                pos: [0.0, 1.0, 0.0, 1.0],
                tex_coord: [0.0, 0.0],
            },
            Vertex {
                pos: [(2.0 * FRAC_PI_3).sin(), (2.0 * FRAC_PI_3).cos(), 0.0, 1.0],
                tex_coord: [1.0, 0.0],
            },
            Vertex {
                pos: [(4.0 * FRAC_PI_3).sin(), (4.0 * FRAC_PI_3).cos(), 0.0, 1.0],
                tex_coord: [0.0, 1.0],
            },
        ];

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create pipeline layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(64),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(glam::Mat4::ZERO.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
            label: None,
        });

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 4,
                    shader_location: 1,
                },
            ],
        }];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &vertex_buffers,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            vertex_buf,
            uniform_buf,
            bind_group,
            pipeline,
        }
    }

    fn render(&mut self, frame: &WgpuFrame<'_, '_, '_>, angle: f32) {
        let aspect_ratio = frame.state.size.0 as f32 / frame.state.size.1 as f32;
        let mut transform =
            glam::Mat4::orthographic_rh(-aspect_ratio, aspect_ratio, -1.0, 1.0, -1.0, 1.0);
        transform *= glam::Mat4::from_rotation_z(angle);
        frame.state.queue.write_buffer(
            &self.uniform_buf,
            0,
            bytemuck::cast_slice(transform.as_ref()),
        );

        let mut encoder = frame
            .state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    // depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            {
                renderpass.push_debug_group("Prepare data for draw.");
                renderpass.set_pipeline(&self.pipeline);
                renderpass.set_bind_group(0, &self.bind_group, &[]);
                renderpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
                renderpass.pop_debug_group();
            }
            renderpass.insert_debug_marker("Draw!");
            renderpass.draw(0..3, 0..1);
        }

        // Submit the command in the queue to execute
        frame.state.queue.submit(Some(encoder.finish()));
    }
}

#[wgame::main]
async fn main(rt: Runtime) {
    env_logger::init();
    println!("Started");

    rt.clone()
        .create_window(WindowAttributes::default(), async move |mut window| {
            println!("Window created");

            let mut state = WgpuState::new(window.inner()).await;
            println!("Surface created");

            let mut scene = TriangleScene::new(&state);
            println!("Scene created");

            let start_time = Instant::now();
            let mut fps = FrameCounter::default();
            while let Some(frame) = window.next_frame().await {
                let angle = (2.0 * PI) * (Instant::now() - start_time).as_secs_f32() / 10.0;
                let frame = state.create_frame(frame);
                scene.render(&frame, angle);
                fps.count();
            }
        })
        .await
        .unwrap()
        .await;
    println!("Closed");
}
