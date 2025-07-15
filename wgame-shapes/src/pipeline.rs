use alloc::borrow::Cow;

use anyhow::Result;

use wgame_gfx::State;

use crate::{
    primitive::{Instance, Vertex},
    shader::{ShaderConfig, ShaderSource},
};

pub fn create_pipeline(state: &State<'_>, config: &ShaderConfig) -> Result<wgpu::RenderPipeline> {
    let device = &state.device();
    let swapchain_format = state.format();

    let shader_source = wgpu::ShaderSource::Wgsl(Cow::Owned(
        ShaderSource::new(
            "shaders/instance.wgsl",
            include_str!("../shaders/instance.wgsl"),
        )?
        .substitute(config)?,
    ));
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("shader"),
        source: shader_source,
    });

    let bind_group_layout = &state.registry().texture_bind_group_layout;

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });

    let vertex_buffers = [Vertex::layout(), Instance::layout()];

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vertex_main"),
            buffers: &vertex_buffers,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fragment_main"),
            compilation_options: Default::default(),
            targets: &[Some(swapchain_format.into())],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    Ok(pipeline)
}
