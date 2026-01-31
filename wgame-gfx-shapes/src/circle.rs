use std::f32::consts::PI;

use glam::{Affine2, Affine3A};
use wgame_gfx::{modifiers::Transformable, types::Transform};
use wgame_shader::Attribute;

use crate::{
    Mesh, Shape, ShapesLibrary, ShapesState,
    pipeline::create_pipeline,
    shader::ShaderConfig,
    shape::{Element, ElementVisitor},
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

#[must_use]
#[derive(Clone)]
pub struct Circle {
    library: ShapesLibrary,
    geometry: Mesh,
    pipeline: wgpu::RenderPipeline,
    inner_radius: f32,
    segment_angle: f32,
    xform: Affine3A,
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

    fn vertices(&self) -> Mesh {
        self.geometry.clone()
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

    fn xform(&self) -> Affine3A {
        self.xform
    }
}

impl Shape for Circle {
    fn library(&self) -> &ShapesLibrary {
        &self.library
    }
    fn for_each_element<V: ElementVisitor>(&self, visitor: &mut V) {
        visitor.visit(self);
    }
}

impl Transformable for Circle {
    fn transform<X: Transform>(&self, xform: X) -> Self {
        Self {
            xform: xform.to_affine3() * self.xform,
            ..self.clone()
        }
    }
}

impl ShapesLibrary {
    pub fn unit_ring(&self, inner_radius: f32) -> Circle {
        Circle {
            library: self.clone(),
            geometry: self.polygon.quad.clone(),
            pipeline: self.circle.ring_pipeline.clone(),
            inner_radius,
            segment_angle: 2.0 * PI,
            xform: Affine3A::IDENTITY,
        }
    }

    pub fn unit_circle(&self) -> Circle {
        Circle {
            library: self.clone(),
            geometry: self.polygon.quad.clone(),
            pipeline: self.circle.circle_pipeline.clone(),
            inner_radius: 0.0,
            segment_angle: 2.0 * PI,
            xform: Affine3A::IDENTITY,
        }
    }
}
