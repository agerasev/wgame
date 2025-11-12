use core::marker::PhantomData;

use half::f16;
use rgb::{ComponentMap, Rgba};

use wgame_gfx::{
    Context, Instance, Object, Resource,
    types::{Color, color},
};

use crate::{Shape, Texture, primitive::InstanceData, renderer::ShapeResource};

#[derive(Clone)]
pub struct Textured<T: Shape> {
    shape: T,
    texture: Texture,
    color: Rgba<f16>,
}

impl<T: Shape> Textured<T> {
    pub fn new(shape: T, texture: Texture) -> Self {
        Self {
            shape,
            texture,
            color: color::WHITE.to_rgba(),
        }
    }

    pub fn color(self, color: impl Color) -> Self {
        Self {
            color: color.to_rgba(),
            ..self
        }
    }
}

impl<T: Shape> Instance for Textured<T> {
    type Resource = ShapeResource<T::Attribute>;

    fn resource(&self) -> Self::Resource {
        ShapeResource {
            order: 0,
            vertices: self.shape.vertices(),
            texture: self.texture.resource(),
            uniforms: self.shape.uniforms(),
            pipeline: self.shape.pipeline(),
            device: self.shape.state().device().clone(),
            _ghost: PhantomData,
        }
    }

    fn store(&self, ctx: &Context, storage: &mut <Self::Resource as Resource>::Storage) {
        storage.instances.push(InstanceData {
            xform: ctx.view * self.shape.xform(),
            tex_xform: self.texture.attribute(),
            color: self.color.map(|x| x.to_f32()),
            custom: self.shape.attribute(),
        });
    }
}

impl<T: Shape> Object for Textured<T> {
    fn collect_into(&self, ctx: &Context, collector: &mut wgame_gfx::Collector) {
        collector.push(ctx, self);
    }
}
