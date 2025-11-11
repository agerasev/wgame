use std::{borrow::Cow, ops::Deref};

use glam::Vec4;
use wgame_gfx_texture::{TextureAtlas, TextureLibrary, TextureState};
use wgpu::util::DeviceExt;

use crate::{Font, FontAtlas, FontTexture};

#[derive(Clone)]
pub struct TextState {
    pub(crate) inner: TextureState,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) pipeline: wgpu::RenderPipeline,
}

impl Deref for TextState {
    type Target = TextureState;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TextState {
    const INSTANCE_COMPONENTS: u32 = 6;

    pub fn new(state: &TextureState) -> Self {
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

        let bind_group_layout = state.bind_group_layout(wgpu::TextureFormat::R8Uint);

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

        Self {
            inner: state.clone(),
            vertex_buffer,
            index_buffer,
            pipeline,
        }
    }
}

#[derive(Clone)]
pub struct TextLibrary {
    state: TextState,
    default_atlas: TextureAtlas<u8>,
}

impl TextLibrary {
    pub fn new(texture: &TextureLibrary) -> Self {
        let state = TextState::new(texture.state());
        Self {
            default_atlas: TextureAtlas::new(
                &state,
                Default::default(),
                wgpu::TextureFormat::R8Uint,
            ),
            state,
        }
    }

    pub fn texture(&self, font: &Font, size: f32) -> FontTexture {
        let atlas = self.default_atlas.inner();
        let font_atlas = FontAtlas::new(&atlas, font, size);
        FontTexture::new(&self.state, &font_atlas, &self.default_atlas)
    }
}
