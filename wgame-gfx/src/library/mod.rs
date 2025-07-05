mod geometry;
mod polygon;
mod texture;

use std::{borrow::Cow, mem::offset_of};

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec4};
use wgpu::util::DeviceExt;

use crate::SharedState;

pub use self::{
    geometry::{Geometry, GeometryExt},
    polygon::Polygon,
    texture::Texture,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    pos: [f32; 4],
    tex_coord: [f32; 2],
}

impl Vertex {
    fn new(pos: Vec4, tex_coord: Vec2) -> Self {
        Self {
            pos: pos.into(),
            tex_coord: tex_coord.into(),
        }
    }
}

/// 2D graphics library
pub struct Library<'a> {
    state: SharedState<'a>,
    quad_vertices: wgpu::Buffer,
    quad_indices: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl<'a> Library<'a> {
    pub fn new(state: &SharedState<'a>) -> Result<Self> {
        let device = &state.device;
        let swapchain_format = state.format;

        // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders.wgsl"))),
        });

        let quad_vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad_vertices"),
            contents: bytemuck::cast_slice(&[
                Vertex::new(Vec4::new(0.0, 0.0, 0.0, 1.0), Vec2::new(0.0, 0.0)),
                Vertex::new(Vec4::new(1.0, 0.0, 0.0, 1.0), Vec2::new(1.0, 0.0)),
                Vertex::new(Vec4::new(0.0, 1.0, 0.0, 1.0), Vec2::new(0.0, 1.0)),
                Vertex::new(Vec4::new(1.0, 1.0, 0.0, 1.0), Vec2::new(1.0, 1.0)),
            ]),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let quad_indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad_indices"),
            contents: bytemuck::cast_slice::<u32, _>(&[0, 1, 2, 2, 1, 3]),
            usage: wgpu::BufferUsages::INDEX,
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
                                size_of::<[[f32; 2]; 3]>() as wgpu::BufferAddress
                            ),
                        },
                        count: None,
                    },
                ],
            });

        let fragment_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
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
                    offset: offset_of!(Vertex, tex_coord) as wgpu::BufferAddress,
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

        Ok(Self {
            state: state.clone(),
            quad_vertices,
            quad_indices,
            pipeline,
        })
    }
}
