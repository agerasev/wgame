use core::marker::PhantomData;

use glam::Affine2;

use wgame_gfx::{
    Instance, InstanceVisitor, Object, Resource,
    modifiers::{Colorable, Transformable},
    types::{Color, Transform},
};

use crate::{
    Shape, Texture,
    resource::ShapeResource,
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
    type Resource = ShapeResource<S::Attribute>;

    fn resource(&self) -> Self::Resource {
        ShapeResource {
            order: 0,
            vertices: self.inner.vertices(),
            texture: self.texture.as_ref().resource(),
            uniforms: self.inner.uniforms(),
            pipeline: self.inner.pipeline(),
            device: self.inner.state().device().clone(),
            _ghost: PhantomData,
        }
    }

    fn store(&self, storage: &mut <Self::Resource as Resource>::Storage) {
        storage.instances.push(InstanceData {
            matrix: self.inner.matrix(),
            tex: self.texture.as_ref().attribute(),
            custom: self.inner.attribute(),
        });
    }
}

impl<V: InstanceVisitor> ElementVisitor for Textured<&mut V> {
    fn visit<S: Element>(&mut self, element: &S) {
        self.inner.visit(Textured {
            inner: element.clone(),
            texture: self.texture.clone(),
        });
    }
}

impl<T: Shape> Object for Textured<T> {
    fn for_each_instance<R: InstanceVisitor>(&self, renderer: &mut R) {
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
    fn multiply_color<C: Color>(&self, color: C) -> Self {
        Self {
            inner: self.inner.clone(),
            texture: self.texture.multiply_color(color),
        }
    }
}
