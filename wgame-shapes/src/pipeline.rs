use alloc::{borrow::Cow, vec::Vec};
use core::mem::{offset_of, size_of};

use anyhow::Result;
use glam::Mat4;

use crate::shader::{ShaderConfig, ShaderSource};

use wgame_gfx::SharedState;

use super::Vertex;

pub fn create_pipeline(state: &SharedState<'_>) -> Result<wgpu::RenderPipeline> {
    create_pipeline_masked(state, &ShaderConfig::default())
}

pub fn create_pipeline_masked(
    state: &SharedState<'_>,
    config: &ShaderConfig,
) -> Result<wgpu::RenderPipeline> {
    let device = &state.device();
    let swapchain_format = state.format();

    let vertex_shader_source = wgpu::ShaderSource::Wgsl(Cow::Owned(
        ShaderSource::new(
            [
                include_str!("../shaders/common.wgsl"),
                include_str!("../shaders/vertex.wgsl"),
            ]
            .join("\n"),
        )?
        .substitute(&ShaderConfig::default())?,
    ));
    let fragment_shader_source = wgpu::ShaderSource::Wgsl(Cow::Owned(
        ShaderSource::new(
            [
                include_str!("../shaders/common.wgsl"),
                include_str!("../shaders/fragment_masked.wgsl"),
            ]
            .join("\n"),
        )?
        .substitute(config)?,
    ));

    let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("vertex"),
        source: vertex_shader_source,
    });
    let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fragment"),
        source: fragment_shader_source,
    });

    let vertex_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            size_of::<Mat4>() as wgpu::BufferAddress
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            size_of::<Mat4>() as wgpu::BufferAddress
                        ),
                    },
                    count: None,
                },
            ],
        });

    let fragment_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &([
                wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            ]
            .into_iter())
            .chain(
                config
                    .uniforms
                    .iter()
                    .map(|uniform| wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            uniform.ty.size() as wgpu::BufferAddress
                        ),
                    }),
            )
            .enumerate()
            .map(|(i, ty)| wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty,
                count: None,
            })
            .collect::<Vec<_>>(),
        });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&vertex_bind_group_layout, &fragment_bind_group_layout],
        push_constant_ranges: &[],
    });

    let vertex_buffers = [wgpu::VertexBufferLayout {
        array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: offset_of!(Vertex, pos) as wgpu::BufferAddress,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: offset_of!(Vertex, local_coord) as wgpu::BufferAddress,
                shader_location: 1,
            },
        ],
    }];

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vertex_shader,
            entry_point: Some("main"),
            buffers: &vertex_buffers,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &fragment_shader,
            entry_point: Some("main"),
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
