use alloc::{string::ToString, vec, vec::Vec};

use anyhow::Result;
use glam::{Affine2, Mat2, Vec2};
use wgpu::util::DeviceExt;

use crate::{
    SharedState,
    library::{
        GeometryExt,
        pipeline::create_pipeline_masked,
        shader::{ScalarType, ShaderConfig, UniformInfo, UniformType},
    },
    object::Vertices,
};

use super::{Geometry, Library};

pub struct CircleRenderer {
    pipeline: wgpu::RenderPipeline,
}

impl CircleRenderer {
    pub fn new(state: &SharedState<'_>) -> Result<Self> {
        let pipeline = create_pipeline_masked(
            state,
            &ShaderConfig {
                mask_stmt: "
                    let c = coord - vec2(0.5, 0.5);
                    let l = 2.0 * length(c);
                    mask = l < 1.0 && l >= inner_radius;
                "
                .to_string(),
                uniforms: vec![UniformInfo {
                    name: "inner_radius".to_string(),
                    ty: UniformType {
                        dims: vec![],
                        item: ScalarType::F32,
                    },
                }],
            },
        )?;

        Ok(Self { pipeline })
    }
}

pub struct Circle<'a> {
    state: SharedState<'a>,
    vertices: wgpu::Buffer,
    indices: Option<wgpu::Buffer>,
    pipeline: wgpu::RenderPipeline,
    inner_radius: f32,
}

impl<'a> Geometry<'a> for Circle<'a> {
    fn state(&self) -> &SharedState<'a> {
        &self.state
    }

    fn vertices(&self) -> Vertices {
        Vertices {
            count: 6,
            vertex_buffer: self.vertices.clone(),
            index_buffer: self.indices.clone(),
        }
    }

    fn uniforms(&self) -> Vec<wgpu::Buffer> {
        vec![
            self.state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("inner_radius"),
                    contents: bytemuck::cast_slice([self.inner_radius].as_ref()),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
        ]
    }

    fn pipeline(&self) -> wgpu::RenderPipeline {
        self.pipeline.clone()
    }
}

impl<'a> Library<'a> {
    pub fn unit_ring(&self, inner_radius: f32) -> impl Geometry<'a> {
        Circle {
            state: self.state.clone(),
            vertices: self.polygon.quad_vertices.clone(),
            indices: Some(self.polygon.quad_indices.clone()),
            pipeline: self.circle.pipeline.clone(),
            inner_radius,
        }
    }

    pub fn ring(&self, pos: Vec2, radius: f32, inner_radius: f32) -> impl Geometry<'a> {
        self.unit_ring(inner_radius / radius)
            .transform(Affine2::from_mat2_translation(
                Mat2::from_diagonal(Vec2::new(radius, radius)),
                pos,
            ))
    }

    pub fn unit_circle(&self) -> impl Geometry<'a> {
        self.unit_ring(0.0)
    }

    pub fn circle(&self, pos: Vec2, radius: f32) -> impl Geometry<'a> {
        self.ring(pos, radius, 0.0)
    }
}
