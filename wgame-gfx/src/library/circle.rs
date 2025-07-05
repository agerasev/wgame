use anyhow::Result;
use glam::{Affine2, Mat2, Vec2};

use crate::{
    SharedState,
    library::{GeometryExt, Polygon, pipeline::create_pipeline_masked},
};

use super::{Geometry, Library};

pub struct CircleRenderer {
    pipeline: wgpu::RenderPipeline,
}

impl CircleRenderer {
    pub fn new(state: &SharedState<'_>) -> Result<Self> {
        let pipeline = create_pipeline_masked(
            state,
            "(x - 0.5) * (x - 0.5) + (y - 0.5) * (y - 0.5) < 0.25",
        )?;

        Ok(Self { pipeline })
    }
}

impl<'a> Library<'a> {
    pub fn unit_circle(&self) -> impl Geometry<'a> {
        Polygon::<4> {
            state: self.state.clone(),
            vertices: self.polygon.quad_vertices.clone(),
            indices: Some(self.polygon.quad_indices.clone()),
            pipeline: self.circle.pipeline.clone(),
        }
        .transform(Affine2::from_mat2_translation(
            Mat2::from_diagonal(Vec2::new(2.0, 2.0)),
            Vec2::new(-1.0, -1.0),
        ))
    }

    pub fn circle(&self, pos: Vec2, radius: f32) -> impl Geometry<'a> {
        self.unit_circle().transform(Affine2::from_mat2_translation(
            Mat2::from_diagonal(Vec2::new(radius, radius)),
            pos,
        ))
    }
}
