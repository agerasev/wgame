use std::f32::consts::PI;

use glam::Affine2;
use wgame_shader::Attribute;

use crate::{
    Shape, ShapesLibrary, ShapesState,
    pipeline::create_pipeline,
    resource::VertexBuffers,
    shader::ShaderConfig,
    shape::{Element, ShapeContext, Visitor},
};

#[derive(Clone, Copy, Attribute)]
pub struct CircleAttrs {
    inner_radius: f32,
    segment_angle: f32,
}

#[derive(Clone, Copy, Attribute)]
pub struct RingVarying {
    inner_radius: f32,
    segment_angle: f32,
    tex_xform: Affine2,
}

#[derive(Clone)]
pub struct CircleLibrary {
    circle_pipeline: wgpu::RenderPipeline,
    ring_pipeline: wgpu::RenderPipeline,
}

impl CircleLibrary {
    pub fn new(state: &ShapesState) -> Self {
        Self {
            circle_pipeline: create_pipeline(
                state,
                &ShaderConfig {
                    instance: CircleAttrs::bindings(),
                    varying: CircleAttrs::bindings(),
                    vertex_source: "
                        output.inner_radius = instance.inner_radius;
                        output.segment_angle = instance.segment_angle;
                    "
                    .to_string(),
                    fragment_color_source: "
                        let c = input.local_coord.xy - vec2(0.5, 0.5);
                        let a = atan2(c.y, c.x);
                        let l = 2.0 * length(c);
                        if (
                            l > 1.0 ||
                            l < input.inner_radius ||
                            a > input.segment_angle
                        ) {
                            discard;
                        }
                    "
                    .to_string(),
                    ..Default::default()
                },
            )
            .expect("Failed to create circle pipeline"),

            ring_pipeline: create_pipeline(
                state,
                &ShaderConfig {
                    instance: CircleAttrs::bindings(),
                    varying: RingVarying::bindings(),
                    vertex_source: "
                        output.inner_radius = instance.inner_radius;
                        output.segment_angle = instance.segment_angle;
                        output.tex_xform_m = instance.tex_xform_m;
                        output.tex_xform_v = instance.tex_xform_v;
                    "
                    .to_string(),
                    fragment_texcoord_source: "
                        let tex_xform = mat3x2<f32>(
                            input.tex_xform_m.xy,
                            input.tex_xform_m.zw,
                            input.tex_xform_v,
                        );
                        let c = input.local_coord.xy - vec2(0.5, 0.5);
                        let l = 2.0 * length(c);
                        let a = atan2(c.y, c.x) + PI;
                        tex_coord = tex_xform * vec3(
                            a / input.segment_angle,
                            (l - input.inner_radius) / (1.0 - input.inner_radius),
                            1.0,
                        );
                    "
                    .to_string(),
                    fragment_color_source: "
                        if (
                            l > 1.0 ||
                            l < input.inner_radius ||
                            a > input.segment_angle
                        ) {
                            discard;
                        }
                    "
                    .to_string(),
                    ..Default::default()
                },
            )
            .expect("Failed to create ring pipeline"),
        }
    }
}

pub struct Circle {
    library: ShapesLibrary,
    vertices: wgpu::Buffer,
    indices: Option<wgpu::Buffer>,
    pipeline: wgpu::RenderPipeline,
    inner_radius: f32,
    segment_angle: f32,
}

impl Circle {
    pub fn inner_radius(self, inner_radius: f32) -> Self {
        Self {
            inner_radius,
            ..self
        }
    }

    pub fn segment(self, angle: f32) -> Self {
        Self {
            segment_angle: angle,
            ..self
        }
    }
}

impl Element for Circle {
    type Attribute = CircleAttrs;

    fn state(&self) -> &ShapesState {
        &self.library.state
    }

    fn vertices(&self) -> VertexBuffers {
        VertexBuffers {
            count: 6,
            vertex_buffer: self.vertices.clone(),
            index_buffer: self.indices.clone(),
        }
    }

    fn attribute(&self) -> Self::Attribute {
        CircleAttrs {
            inner_radius: self.inner_radius,
            segment_angle: self.segment_angle,
        }
    }

    fn pipeline(&self) -> wgpu::RenderPipeline {
        self.pipeline.clone()
    }
}

impl Shape for Circle {
    fn library(&self) -> &ShapesLibrary {
        &self.library
    }
    fn visit<V: Visitor>(&self, ctx: ShapeContext, visitor: &mut V) {
        visitor.apply(ctx, self);
    }
}

impl ShapesLibrary {
    pub fn unit_ring(&self, inner_radius: f32) -> Circle {
        Circle {
            library: self.clone(),
            vertices: self.polygon.quad_vertices.clone(),
            indices: Some(self.polygon.quad_indices.clone()),
            pipeline: self.circle.ring_pipeline.clone(),
            inner_radius,
            segment_angle: 2.0 * PI,
        }
    }

    pub fn unit_circle(&self) -> Circle {
        Circle {
            library: self.clone(),

            vertices: self.polygon.quad_vertices.clone(),
            indices: Some(self.polygon.quad_indices.clone()),
            pipeline: self.circle.circle_pipeline.clone(),
            inner_radius: 0.0,
            segment_angle: 2.0 * PI,
        }
    }
}
