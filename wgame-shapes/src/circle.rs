use alloc::string::ToString;

use anyhow::Result;
use glam::{Affine2, Mat2, Vec2};
use wgame_macros::{Attributes, StoreBytes};

use wgame_gfx::{State, Vertices};

use crate::{
    Library, Shape, ShapeExt, attributes::Attributes, pipeline::create_pipeline,
    shader::ShaderConfig,
};

#[derive(Clone, Copy, StoreBytes, Attributes)]
#[bytes_mod(wgame_gfx::bytes)]
#[attributes_mod(crate::attributes)]
pub struct CircleAttrs {
    inner_radius: f32,
}

pub struct CircleRenderer {
    pipeline: wgpu::RenderPipeline,
}

impl CircleRenderer {
    pub fn new(state: &State<'_>) -> Result<Self> {
        let pipeline = create_pipeline(
            state,
            &ShaderConfig {
                fragment_modifier: "
                    let c = coord - vec2(0.5, 0.5);
                    let l = 2.0 * length(c);
                    if (l > 1.0 || l < vertex.custom_inner_radius) {
                        discard;
                    }
                "
                .to_string(),
                instances: CircleAttrs::attributes().with_prefix("custom"),
                ..Default::default()
            },
        )?;

        Ok(Self { pipeline })
    }
}

pub struct Circle<'a> {
    state: State<'a>,
    vertices: wgpu::Buffer,
    indices: Option<wgpu::Buffer>,
    pipeline: wgpu::RenderPipeline,
    inner_radius: f32,
}

impl<'a> Circle<'a> {
    fn new(
        state: State<'a>,
        vertices: wgpu::Buffer,
        indices: Option<wgpu::Buffer>,
        pipeline: wgpu::RenderPipeline,
        inner_radius: f32,
    ) -> Self {
        Self {
            state,
            vertices,
            indices,
            pipeline,
            inner_radius,
        }
    }
}

impl<'a> Shape<'a> for Circle<'a> {
    type Attributes = CircleAttrs;

    fn state(&self) -> &State<'a> {
        &self.state
    }

    fn vertices(&self) -> Vertices {
        Vertices {
            count: 6,
            vertex_buffer: self.vertices.clone(),
            index_buffer: self.indices.clone(),
        }
    }

    fn attributes(&self) -> Self::Attributes {
        CircleAttrs {
            inner_radius: self.inner_radius,
        }
    }

    fn pipeline(&self) -> wgpu::RenderPipeline {
        self.pipeline.clone()
    }
}

impl<'a> Library<'a> {
    pub fn unit_ring(&self, inner_radius: f32) -> impl Shape<'a> {
        Circle::new(
            self.state.clone(),
            self.polygon.quad_vertices.clone(),
            Some(self.polygon.quad_indices.clone()),
            self.circle.pipeline.clone(),
            inner_radius,
        )
    }

    pub fn ring(&self, pos: Vec2, radius: f32, inner_radius: f32) -> impl Shape<'a> {
        self.unit_ring(inner_radius / radius)
            .transform(Affine2::from_mat2_translation(
                Mat2::from_diagonal(Vec2::new(radius, radius)),
                pos,
            ))
    }

    pub fn unit_circle(&self) -> impl Shape<'a> {
        self.unit_ring(0.0)
    }

    pub fn circle(&self, pos: Vec2, radius: f32) -> impl Shape<'a> {
        self.ring(pos, radius, 0.0)
    }
}
