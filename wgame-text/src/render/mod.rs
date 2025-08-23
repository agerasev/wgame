mod texture;

use std::{borrow::Cow, ops::Deref};

use anyhow::Result;
use glam::{Mat4, Vec4};
use wgpu::util::DeviceExt;

use wgame_gfx::{Graphics, Renderer, Resources, utils::Ordered};

pub use texture::TexturedFont;

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
            wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../shaders/text.wgsl")));
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
                    format: wgpu::VertexFormat::Float32x4,
                }],
            },
            wgpu::VertexBufferLayout {
                array_stride: 4 * 4 * Self::INSTANCE_COMPONENTS as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &(0..Self::INSTANCE_COMPONENTS)
                    .map(|i| wgpu::VertexAttribute {
                        shader_location: i + 1,
                        offset: 4 * 4 * i as u64,
                        format: wgpu::VertexFormat::Float32x4,
                    })
                    .collect::<Vec<_>>(),
            },
        ];

        let vertex_buffer = state
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("glyph_vertices"),
                contents: bytemuck::cast_slice(&[
                    Vec4::new(0.0, 0.0, 0.0, 1.0),
                    Vec4::new(0.0, -1.0, 0.0, 1.0),
                    Vec4::new(1.0, 0.0, 0.0, 1.0),
                    Vec4::new(1.0, -1.0, 0.0, 1.0),
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
pub struct TextResources {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture: TexturedFont,
    pipeline: wgpu::RenderPipeline,
    device: wgpu::Device,
}

impl TextResources {
    pub fn new(font: &TexturedFont) -> Self {
        let library = &font.library;
        let pipeline = library.pipeline.clone();

        Self {
            vertex_buffer: library.vertex_buffer.clone(),
            index_buffer: library.index_buffer.clone(),
            pipeline,
            texture: font.clone(),
            device: library.device().clone(),
        }
    }
}

pub struct GlyphInstance {
    pub xform: Mat4,
    pub tex_coord: Vec4,
    pub color: Vec4,
}

#[derive(Default)]
pub struct TextStorage {
    pub(crate) instances: Vec<GlyphInstance>,
}

pub struct TextRenderer {
    resources: TextResources,
    bind_group: wgpu::BindGroup,
    instance_buffer: wgpu::Buffer,
    instance_count: u32,
}

impl Resources for TextResources {
    type Storage = TextStorage;
    type Renderer = TextRenderer;

    fn new_storage(&self) -> Self::Storage {
        TextStorage::default()
    }
    fn make_renderer(&self, storage: &Self::Storage) -> Result<Self::Renderer> {
        let mut bytes = Vec::new();
        for instance in &storage.instances {
            bytes.extend_from_slice(bytemuck::cast_slice(&[instance.xform]));
            bytes.extend_from_slice(bytemuck::cast_slice(&[instance.tex_coord]));
            bytes.extend_from_slice(bytemuck::cast_slice(&[instance.color]));
        }
        let instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instances"),
                contents: &bytes,
                usage: wgpu::BufferUsages::VERTEX,
            });

        let texture_view = self.texture.sync().unwrap();
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            }],
            label: None,
        });

        Ok(TextRenderer {
            resources: self.clone(),
            bind_group,
            instance_buffer,
            instance_count: storage.instances.len() as u32,
        })
    }
}
impl Ordered for TextResources {
    fn order(&self) -> i64 {
        // Text is rendered over other shapes by default
        1 << 16
    }
}

impl Renderer for TextRenderer {
    fn draw(&self, pass: &mut wgpu::RenderPass) -> Result<()> {
        pass.push_debug_group("prepare");
        pass.set_pipeline(&self.resources.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.resources.vertex_buffer.slice(..));
        pass.set_index_buffer(
            self.resources.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.pop_debug_group();

        pass.insert_debug_marker("draw");
        pass.draw_indexed(0..6, 0, 0..self.instance_count);

        Ok(())
    }
}
