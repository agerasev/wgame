use core::marker::PhantomData;

use glam::{Mat4, Vec3};
use half::f16;
use rgb::{ComponentMap, Rgba};

use wgame_gfx::{
    Camera, Instance, Object, Renderer, Resource,
    types::{Color, color},
};

use crate::{Shape, Texture, renderer::ShapeResource, shader::InstanceData};

#[derive(Clone, Debug)]
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
    type Context = Camera;

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

    fn store(&self, camera: &Camera, storage: &mut <Self::Resource as Resource>::Storage) {
        storage.instances.push(InstanceData {
            xform: camera.view
                * self.shape.matrix()
                * Mat4::from_scale(Vec3::new(1.0, if camera.y_flip { -1.0 } else { 1.0 }, 1.0)),
            tex_xform: self.texture.attribute(),
            color: self.color.map(|x| x.to_f32()),
            custom: self.shape.attribute(),
        });
    }
}

impl<T: Shape> Object for Textured<T> {
    type Context = Camera;

    fn draw<R: Renderer<Self::Context>>(&self, visitor: &mut R) {
        visitor.insert(self);
    }
}
