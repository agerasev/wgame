use std::{f32::consts::PI, marker::PhantomData};

use glam::{Affine2, Affine3A, Vec3};
use wgame_gfx::{
    Camera, Instance, Object, delegate_transformable, impl_object_for_instance, impl_transformable,
    prelude::Transformable, types::Transform,
};
use wgame_gfx_texture::Texture;
use wgame_shader::Attribute;

use crate::{
    Mesh, Shape, ShapesLibrary, ShapesState, impl_textured,
    pipeline::create_pipeline,
    render::{ShapeResource, ShapeStorage},
    shader::{InstanceData, ShaderConfig},
    shape::{ShapeFill, ShapeStroke},
};

#[derive(Clone, Copy, Attribute)]
pub struct CircleAttrs {
    inner_radius: f32,
    sector_angle: f32,
}

#[derive(Attribute)]
struct RingVarying {
    inner_radius: f32,
    sector_angle: f32,
    tex_xform: Affine2,
}

#[derive(Clone)]
pub struct CircleLibrary {
    fill: wgpu::RenderPipeline,
    stroke: wgpu::RenderPipeline,
}

impl CircleLibrary {
    pub fn new(state: &ShapesState) -> Self {
        Self {
            fill: create_pipeline(
                state,
                &ShaderConfig {
                    instance: CircleAttrs::bindings(),
                    varying: CircleAttrs::bindings(),
                    vertex_source: "
                        output.inner_radius = instance.inner_radius;
                        output.sector_angle = instance.sector_angle;
                    "
                    .to_string(),
                    fragment_color_source: "
                        let c = input.local_coord.xy - vec2(0.5, 0.5);
                        let a = atan2(c.y, c.x);
                        let l = 2.0 * length(c);
                        if (
                            l > 1.0 ||
                            l < input.inner_radius ||
                            a > input.sector_angle
                        ) {
                            discard;
                        }
                    "
                    .to_string(),
                    ..Default::default()
                },
            )
            .expect("Failed to create circle pipeline"),

            stroke: create_pipeline(
                state,
                &ShaderConfig {
                    instance: CircleAttrs::bindings(),
                    varying: RingVarying::bindings(),
                    vertex_source: "
                        output.inner_radius = instance.inner_radius;
                        output.sector_angle = instance.sector_angle;
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
                            a / input.sector_angle,
                            (l - input.inner_radius) / (1.0 - input.inner_radius),
                            1.0,
                        );
                    "
                    .to_string(),
                    fragment_color_source: "
                        if (
                            l > 1.0 ||
                            l < input.inner_radius ||
                            a > input.sector_angle
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
    fill: wgpu::RenderPipeline,
    stroke: wgpu::RenderPipeline,
    inner_radius: f32,
    sector_angle: f32,
    xform: Affine3A,
}

impl Circle {
    pub fn inner_radius(&self, inner_radius: f32) -> Self {
        Self {
            inner_radius,
            ..self.clone()
        }
    }

    pub fn sector(&self, angle: f32) -> Self {
        Self {
            sector_angle: angle,
            ..self.clone()
        }
    }

    fn attribute(&self) -> CircleAttrs {
        CircleAttrs {
            inner_radius: self.inner_radius,
            sector_angle: self.sector_angle,
        }
    }
}

impl Shape for Circle {
    fn library(&self) -> &ShapesLibrary {
        &self.library
    }
}

impl ShapeFill for Circle {
    type Fill = CircleFill;

    fn fill_texture(&self, texture: &Texture) -> Self::Fill {
        CircleFill {
            shape: self.clone(),
            texture: texture.clone(),
        }
    }
}

impl ShapeStroke for Circle {
    type Stroke = CircleStroke;

    fn stroke_texture(&self, line_width: f32, texture: &Texture) -> Self::Stroke {
        let half_width = line_width / 2.0;
        CircleStroke {
            shape: self
                .inner_radius((1.0 - half_width) / (1.0 + half_width))
                .transform(Affine3A::from_scale(Vec3::splat(1.0 + half_width))),
            texture: texture.clone(),
        }
    }
}

impl_transformable!(Circle, xform);

#[must_use]
#[derive(Clone)]
pub struct CircleFill {
    shape: Circle,
    texture: Texture,
}

impl CircleFill {
    pub fn inner_radius(&self, inner_radius: f32) -> Self {
        Self {
            shape: self.shape.inner_radius(inner_radius),
            ..self.clone()
        }
    }

    pub fn sector(&self, angle: f32) -> Self {
        Self {
            shape: self.shape.sector(angle),
            ..self.clone()
        }
    }
}

impl Instance for CircleFill {
    type Context = Camera;
    type Resource = ShapeResource<CircleAttrs>;
    type Storage = ShapeStorage<CircleAttrs>;

    fn resource(&self) -> Self::Resource {
        ShapeResource {
            vertices: self.shape.geometry.clone(),
            texture: self.texture.resource(),
            uniforms: None,
            pipeline: self.shape.fill.clone(),
            device: self.shape.library.state().device().clone(),
            _ghost: PhantomData,
        }
    }

    fn new_storage(&self) -> Self::Storage {
        ShapeStorage::new(self.resource())
    }

    fn store(&self, storage: &mut Self::Storage) {
        storage.instances.push(InstanceData {
            matrix: self.shape.xform.to_mat4(),
            tex: self.texture.attribute(),
            custom: self.shape.attribute(),
        });
    }
}

impl_object_for_instance!(CircleFill);
delegate_transformable!(CircleFill, shape);
impl_textured!(CircleFill, texture);

#[must_use]
#[derive(Clone)]
pub struct CircleStroke {
    shape: Circle,
    texture: Texture,
}

impl CircleStroke {
    pub fn sector(&self, angle: f32) -> Self {
        Self {
            shape: self.shape.sector(angle),
            ..self.clone()
        }
    }
}

impl Instance for CircleStroke {
    type Context = Camera;
    type Resource = ShapeResource<CircleAttrs>;
    type Storage = ShapeStorage<CircleAttrs>;

    fn resource(&self) -> Self::Resource {
        ShapeResource {
            vertices: self.shape.geometry.clone(),
            texture: self.texture.resource(),
            uniforms: None,
            pipeline: self.shape.stroke.clone(),
            device: self.shape.library.state().device().clone(),
            _ghost: PhantomData,
        }
    }

    fn new_storage(&self) -> Self::Storage {
        ShapeStorage::new(self.resource())
    }

    fn store(&self, storage: &mut Self::Storage) {
        storage.instances.push(InstanceData {
            matrix: self.shape.xform.to_mat4(),
            tex: self.texture.attribute(),
            custom: self.shape.attribute(),
        });
    }
}

impl_object_for_instance!(CircleStroke);
delegate_transformable!(CircleStroke, shape);
impl_textured!(CircleStroke, texture);

impl ShapesLibrary {
    pub fn unit_circle(&self) -> Circle {
        Circle {
            library: self.clone(),
            geometry: self.polygon.quad.clone(),
            fill: self.circle.fill.clone(),
            stroke: self.circle.stroke.clone(),
            inner_radius: 0.0,
            sector_angle: 2.0 * PI,
            xform: Affine3A::IDENTITY,
        }
    }
}
