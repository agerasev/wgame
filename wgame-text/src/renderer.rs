use alloc::{borrow::Cow, vec::Vec};

use anyhow::Result;
use wgpu::util::DeviceExt;

use wgame_gfx::{Graphics, Renderer};

#[derive(Default)]
pub struct TextStorage {
    pub count: u32,
    pub data: Vec<u8>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextRenderer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    device: wgpu::Device,
}

impl TextRenderer {
    pub fn new(state: &Graphics) -> Result<Self> {
        let device = state.device().clone();
        let swapchain_format = state.format();

        let shader_source =
            wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/text.wgsl")));
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("text_shader"),
            source: shader_source,
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

        let vertex_buffers = [
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
                array_stride: 5 * 4 * 4,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &(0..5)
                    .map(|i| wgpu::VertexAttribute {
                        shader_location: i + 1,
                        offset: i as u64 * 4 * 4,
                        format: wgpu::VertexFormat::Uint32x4,
                    })
                    .collect::<Vec<_>>(),
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
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Ok(Self { pipeline, device })
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
