use std::{borrow::Cow, mem::offset_of};

use anyhow::Result;
use glam::{Affine3A, Mat3, Mat4, Vec2, Vec3, Vec4};
use wgpu::util::DeviceExt;

use crate::{SharedState, Transformed, object::Vertices, types::Position};

use super::{Geometry, GeometryExt, Library, Vertex};

pub struct PolygonRenderer {
    quad_vertices: wgpu::Buffer,
    quad_indices: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl PolygonRenderer {
    pub fn new(state: &SharedState<'_>) -> Result<Self> {
        let device = &state.device;
        let swapchain_format = state.format;

        let vertex_shader_source = wgpu::ShaderSource::Wgsl(Cow::Owned(
            [
                include_str!("../../shaders/common.wgsl"),
                include_str!("../../shaders/vertex.wgsl"),
            ]
            .join("\n"),
        ));
        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("vertex"),
            source: vertex_shader_source,
        });
        let fragment_shader_source = wgpu::ShaderSource::Wgsl(Cow::Owned(
            [
                include_str!("../../shaders/common.wgsl"),
                include_str!("../../shaders/fragment.wgsl"),
            ]
            .join("\n"),
        ));
        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fragment"),
            source: fragment_shader_source,
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

        Ok(Self {
            quad_vertices,
            quad_indices,
            pipeline,
        })
    }
}

pub struct Polygon<'a, const N: u32> {
    pub(crate) state: SharedState<'a>,
    pub(crate) vertices: wgpu::Buffer,
    pub(crate) indices: Option<wgpu::Buffer>,
    pub(crate) pipeline: wgpu::RenderPipeline,
}

impl<'a, const N: u32> Geometry<'a> for Polygon<'a, N> {
    fn state(&self) -> &SharedState<'a> {
        &self.state
    }

    fn vertices(&self) -> Vertices {
        Vertices {
            count: 3 * (N - 2),
            vertex_buffer: self.vertices.clone(),
            index_buffer: self.indices.clone(),
        }
    }

    fn transformation(&self) -> Mat4 {
        Mat4::IDENTITY
    }

    fn pipeline(&self) -> wgpu::RenderPipeline {
        self.pipeline.clone()
    }
}

impl<'a> Library<'a> {
    pub fn triangle(&self, a: impl Position, b: impl Position, c: impl Position) -> Polygon<'a, 3> {
        let vertices = self
            .state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("triangle_vertices"),
                contents: bytemuck::cast_slice(&[
                    Vertex::new(a.to_xyzw(), Vec2::new(0.0, 0.0)),
                    Vertex::new(b.to_xyzw(), Vec2::new(1.0, 0.0)),
                    Vertex::new(c.to_xyzw(), Vec2::new(0.0, 1.0)),
                ]),
                usage: wgpu::BufferUsages::VERTEX,
            });
        Polygon {
            state: self.state.clone(),
            vertices,
            indices: None,
            pipeline: self.polygons.pipeline.clone(),
        }
    }

    pub fn unit_quad(&self) -> Polygon<'a, 4> {
        Polygon {
            state: self.state.clone(),
            vertices: self.polygons.quad_vertices.clone(),
            indices: Some(self.polygons.quad_indices.clone()),
            pipeline: self.polygons.pipeline.clone(),
        }
    }

    pub fn quad(&self, a: Vec2, b: Vec2) -> Transformed<Polygon<'a, 4>> {
        let min = a.min(b);
        let max = a.max(b);
        let size = max - min;
        let affine = Affine3A::from_mat3_translation(
            Mat3::from_cols(
                Vec3::new(size.x, 0.0, 0.0),
                Vec3::new(0.0, size.y, 0.0),
                Vec3::Z,
            ),
            Vec3::from((a, 0.0)),
        );
        self.unit_quad().transform(affine)
    }
}
