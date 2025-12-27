use core::marker::PhantomData;

use glam::Affine2;

use wgame_gfx::{
    Camera, Instance, InstanceVisitor, Object,
    modifiers::{Colorable, Transformable},
    types::{Color, Transform},
};

use crate::{
    Shape, Texture,
    render::{ShapeResource, ShapeStorage},
    shader::InstanceData,
    shape::{Element, ElementVisitor},
};

#[derive(Clone, Debug)]
pub struct Textured<S> {
    pub inner: S,
    pub texture: Texture,
}

impl<S> Textured<S> {
    pub fn new(inner: S, texture: Texture) -> Self {
        Self { inner, texture }
    }
}

impl<S: Clone> Textured<S> {
    pub fn tranform_texcoord(&self, tex_xform: Affine2) -> Self {
        Self {
            texture: self.texture.transform_coord(tex_xform),
            ..(*self).clone()
        }
    }
}

impl<S: Element> Instance for Textured<S> {
    type Context = Camera;
    type Resource = ShapeResource<S::Attribute>;
    type Storage = ShapeStorage<S::Attribute>;

    fn resource(&self) -> Self::Resource {
        ShapeResource {
            vertices: self.inner.vertices(),
            texture: self.texture.as_ref().resource(),
            uniforms: self.inner.uniforms(),
            pipeline: self.inner.pipeline(),
            device: self.inner.state().device().clone(),
            _ghost: PhantomData,
        }
    }

    fn new_storage(&self) -> Self::Storage {
        ShapeStorage::new(self.resource())
    }

    fn store(&self, storage: &mut Self::Storage) {
        storage.instances.push(InstanceData {
            matrix: self.inner.xform().to_mat4(),
            tex: self.texture.as_ref().attribute(),
            custom: self.inner.attribute(),
        });
    }
}

impl<V: InstanceVisitor<Camera>> ElementVisitor for Textured<&mut V> {
    fn visit<S: Element>(&mut self, element: &S) {
        self.inner.visit(&Textured {
            inner: element.clone(),
            texture: self.texture.clone(),
        });
    }
}

impl<T: Shape> Object for Textured<T> {
    type Context = Camera;
    fn for_each_instance<R: InstanceVisitor<Camera>>(&self, renderer: &mut R) {
        self.inner.for_each_element(&mut Textured {
            inner: renderer,
            texture: self.texture.clone(),
        });
    }
}

impl<S: Shape> Transformable for Textured<S> {
    fn transform<X: Transform>(&self, xform: X) -> Self {
        Self {
            inner: self.inner.transform(xform),
            ..(*self).clone()
        }
    }
}

impl<S: Shape> Colorable for Textured<S> {
    fn mul_color<C: Color>(&self, color: C) -> Self {
        Self {
            inner: self.inner.clone(),
            texture: self.texture.multiply_color(color),
        }
    }
}
