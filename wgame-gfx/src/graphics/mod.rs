pub mod triangle;

use std::{borrow::Cow, f32::consts::FRAC_PI_3, mem::offset_of};

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use glam::{Mat2, Vec2};
use wgpu::util::DeviceExt;

use crate::{graphics::triangle::Triangle, surface::Surface};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    pos: Vec2,
    tex_coord: Vec2,
}

/// 2D graphics
pub struct Graphics<'a> {
    device: &'a wgpu::Device,
    triangle_vertices: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl<'a> Graphics<'a> {
    pub fn new(surface: &'a Surface<'_>) -> Result<Self> {
        let device = &surface.device;
        let swapchain_format = surface.format;

        // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let triangle_vertices = [
            Vertex {
                pos: Vec2::new(0.0, 1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            },
            Vertex {
                pos: Vec2::new((2.0 * FRAC_PI_3).sin(), (2.0 * FRAC_PI_3).cos()),
                tex_coord: Vec2::new(1.0, 0.0),
            },
            Vertex {
                pos: Vec2::new((4.0 * FRAC_PI_3).sin(), (4.0 * FRAC_PI_3).cos()),
                tex_coord: Vec2::new(0.0, 1.0),
            },
        ];

        let triangle_vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&triangle_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        size_of::<Mat2>() as wgpu::BufferAddress
                    ),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
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
            device,
            triangle_vertices,
            pipeline,
        })
    }

    pub fn triangle(&self) -> Triangle<'_> {
        Triangle {
            device: self.device,
            vertex_buffer: &self.triangle_vertices,
            pipeline: &self.pipeline,
        }
    }
}
