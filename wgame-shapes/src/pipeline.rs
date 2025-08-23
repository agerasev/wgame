use alloc::borrow::Cow;

use anyhow::Result;

use crate::{
    LibraryState,
    attributes::Attributes,
    primitive::{InstanceData, VertexData},
    shader::{ShaderConfig, ShaderSource},
};

pub fn create_pipeline(
    state: &LibraryState,
    config: &ShaderConfig,
) -> Result<wgpu::RenderPipeline> {
    let device = state.device();
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

    let bind_group_layout = &state.texture_bind_group_layout;

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });

    let vertex_attributes = VertexData::attributes();
    let instance_attributes = InstanceData::<()>::attributes().chain(config.instances.clone());
    let vertex_buffers = [
        wgpu::VertexBufferLayout {
            array_stride: vertex_attributes.size(),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attributes.layout(0)?,
        },
        wgpu::VertexBufferLayout {
            array_stride: instance_attributes.size(),
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &instance_attributes.layout(vertex_attributes.count())?,
        },
    ];

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
            targets: &[Some(wgpu::ColorTargetState {
                format: swapchain_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    Ok(pipeline)
}
