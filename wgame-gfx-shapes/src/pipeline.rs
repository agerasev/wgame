use std::borrow::Cow;

use anyhow::Result;
use wgame_shader::{Attribute, ShaderSource};

use crate::{
    ShapesState,
    shader::{InstanceData, ShaderConfig, Vertex},
};

#[derive(Clone)]
pub struct Pipelines([wgpu::RenderPipeline; 3]);
impl Pipelines {
    pub fn get(&self, mode: wgame_gfx::DepthMode) -> wgpu::RenderPipeline {
        self.0[mode as usize].clone()
    }
}
pub fn create_pipeline(state: &ShapesState, config: &ShaderConfig) -> Result<Pipelines> {
    use wgame_gfx::DepthMode;
    Ok(Pipelines([
        create_one(state, config, DepthMode::ReadWrite)?,
        create_one(state, config, DepthMode::ReadOnly)?,
        create_one(state, config, DepthMode::Overlay)?,
    ]))
}
fn create_one(
    state: &ShapesState,
    config: &ShaderConfig,
    depth: wgame_gfx::DepthMode,
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

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[
            Some(state.camera_bind_group_layout()),
            Some(&state.texture().float_bind_group_layout),
        ],
        immediate_size: 0,
    });

    let vertex_attributes = Vertex::bindings();
    let instance_attributes = InstanceData::<()>::bindings().chain(config.instance.clone());
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
            buffers: &vertex_buffers.map(Some),
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
        depth_stencil: Some(depth.state()),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    Ok(pipeline)
}
