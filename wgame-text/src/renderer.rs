use std::{borrow::Cow, ops::Deref};

use anyhow::Result;
use glam::Vec4;
use wgpu::util::DeviceExt;

use wgame_gfx::{Graphics, Renderer};

use crate::FontAtlas;

#[derive(Default)]
pub struct TextStorage {
    pub count: u32,
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub struct TextLibrary {
    state: Graphics,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl Deref for TextLibrary {
    type Target = Graphics;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl TextLibrary {
    const INSTANCE_COMPONENTS: u32 = 6;

    pub fn new(state: &Graphics) -> Result<Self> {
        let device = state.device().clone();
        let swapchain_format = state.format();

        let shader_source =
            wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/text.wgsl")));
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("text_shader"),
            source: shader_source,
        });

        let vertex_buffers_layout = [
            wgpu::VertexBufferLayout {
                array_stride: 4 * 4,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    shader_location: 0,
                    offset: 0,
                    format: wgpu::VertexFormat::Uint32x4,
                }],
            },
            wgpu::VertexBufferLayout {
                array_stride: 4 * 4 * Self::INSTANCE_COMPONENTS as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &(0..Self::INSTANCE_COMPONENTS)
                    .map(|i| wgpu::VertexAttribute {
                        shader_location: i + 1,
                        offset: 4 * 4 * i as u64,
                        format: wgpu::VertexFormat::Uint32x4,
                    })
                    .collect::<Vec<_>>(),
            },
        ];

        let vertex_buffer = state
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("glyph_vertices"),
                contents: bytemuck::cast_slice(&[
                    Vec4::new(-1.0, -1.0, 0.0, 1.0),
                    Vec4::new(1.0, -1.0, 0.0, 1.0),
                    Vec4::new(-1.0, 1.0, 0.0, 1.0),
                    Vec4::new(1.0, 1.0, 0.0, 1.0),
                ]),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = state
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("glyph_indices"),
                contents: bytemuck::cast_slice::<u32, _>(&[0, 1, 2, 2, 1, 3]),
                usage: wgpu::BufferUsages::INDEX,
            });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("text_bind_group"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Uint,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                buffers: &vertex_buffers_layout,
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

        Ok(Self {
            state: state.clone(),
            vertex_buffer,
            index_buffer,
            pipeline,
        })
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextRenderer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl TextRenderer {
    pub fn new(library: &TextLibrary, font: &FontAtlas) -> Self {
        let pipeline = library.pipeline.clone();
        let bind_group = library
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                }],
                label: None,
            });
        Self {
            vertex_buffer: library.vertex_buffer.clone(),
            index_buffer: library.index_buffer.clone(),
            pipeline,
            bind_group,
        }
    }
}

impl Renderer for TextRenderer {
    type Storage = TextStorage;

    fn new_storage(&self) -> Self::Storage {
        TextStorage::default()
    }
    fn draw(&self, instances: &Self::Storage, pass: &mut wgpu::RenderPass) -> Result<()> {
        let instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instances"),
                contents: &instances.data,
                usage: wgpu::BufferUsages::VERTEX,
            });

        {
            pass.push_debug_group("prepare");
            pass.set_pipeline(&self.pipeline);
            for (i, bind_group) in self.uniforms.iter().enumerate() {
                pass.set_bind_group(i as u32, bind_group, &[]);
            }
            pass.set_vertex_buffer(0, self.vertices.vertex_buffer.slice(..));
            if let Some(index_buffer) = &self.vertices.index_buffer {
                pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            }
            pass.set_vertex_buffer(1, instance_buffer.slice(..));
            pass.pop_debug_group();
        }

        pass.insert_debug_marker("draw");
        if self.vertices.index_buffer.is_some() {
            pass.draw_indexed(0..self.vertices.count, 0, 0..instances.count);
        } else {
            pass.draw(0..self.vertices.count, 0..instances.count);
        }

        Ok(())
    }
}
