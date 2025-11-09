use alloc::string::ToString;

use glam::{Affine2, Mat2, Vec2};
use wgame_macros::{Attributes, StoreBytes};

use crate::{
    Shape, ShapeExt, ShapesLibrary, ShapesState, attributes::Attributes, pipeline::create_pipeline,
    renderer::VertexBuffers, shader::ShaderConfig,
};

#[derive(Clone, Copy, StoreBytes, Attributes)]
#[bytes_mod(crate::bytes)]
#[attributes_mod(crate::attributes)]
pub struct CircleAttrs {
    inner_radius: f32,
}

#[derive(Clone)]
pub struct CircleLibrary {
    pipeline: wgpu::RenderPipeline,
}

impl CircleLibrary {
    pub fn new(state: &ShapesState) -> Self {
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
        )
        .expect("Failed to create circle pipeline");

        Self { pipeline }
    }
}

pub struct Circle {
    state: ShapesLibrary,
    vertices: wgpu::Buffer,
    indices: Option<wgpu::Buffer>,
    pipeline: wgpu::RenderPipeline,
    inner_radius: f32,
}

impl Shape for Circle {
    type Attributes = CircleAttrs;

    fn state(&self) -> &ShapesLibrary {
        &self.state
    }

    fn vertices(&self) -> VertexBuffers {
        VertexBuffers {
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

impl ShapesLibrary {
    pub fn unit_ring(&self, inner_radius: f32) -> impl Shape {
        Circle {
            state: self.clone(),
            vertices: self.polygon.quad_vertices.clone(),
            indices: Some(self.polygon.quad_indices.clone()),
            pipeline: self.circle.pipeline.clone(),
            inner_radius,
        }
    }

    pub fn ring(&self, pos: Vec2, radius: f32, inner_radius: f32) -> impl Shape {
        self.unit_ring(inner_radius / radius)
            .transform(Affine2::from_mat2_translation(
                Mat2::from_diagonal(Vec2::new(radius, radius)),
                pos,
            ))
    }

    pub fn unit_circle(&self) -> impl Shape {
        self.unit_ring(0.0)
    }

    pub fn circle(&self, pos: Vec2, radius: f32) -> impl Shape {
        self.ring(pos, radius, 0.0)
    }
}
