use core::marker::PhantomData;

use glam::{Mat4, Vec3};

use wgame_gfx::{
    Camera, Instance, Object, Renderer, Resource,
    modifiers::{Colored, Transformed},
    types::Color,
};

use crate::{
    Shape, Texture,
    resource::ShapeResource,
    shader::InstanceData,
    shape::{Element, ShapeContext, Visitor},
};

#[derive(Clone, Debug)]
pub struct Textured<S, T: AsRef<Texture> = Texture> {
    pub inner: S,
    pub texture: T,
}

impl<S, T: AsRef<Texture>> Textured<S, T> {
    pub fn new(inner: S, texture: T) -> Self {
        Self { inner, texture }
    }
}

impl<S: Element, T: AsRef<Texture>> Instance for Textured<&S, T> {
    type Resource = ShapeResource<S::Attribute>;
    type Context = Camera;

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

    fn store(&self, camera: &Camera, storage: &mut <Self::Resource as Resource>::Storage) {
        storage.instances.push(InstanceData {
            xform: camera.view
                * Mat4::from_scale(Vec3::new(1.0, if camera.y_flip { -1.0 } else { 1.0 }, 1.0)),
            tex_xform: self.texture.as_ref().attribute(),
            color: camera.color.to_vec4(),
            custom: self.inner.attribute(),
        });
    }
}

impl<R: Renderer, T: AsRef<Texture>> Visitor for Textured<&mut R, T> {
    fn apply<S: Element>(&mut self, ctx: ShapeContext, element: &S) {
        self.inner.insert(Transformed::new(
            Colored::new(
                Textured {
                    inner: element,
                    texture: self.texture.as_ref().transform_coord(ctx.tex_xform),
                },
                ctx.color,
            ),
            ctx.xform,
        ));
    }
}

impl<T: Shape> Object for Textured<T> {
    type Context = Camera;

    fn draw<R: Renderer>(&self, renderer: &mut R) {
        self.inner.visit(
            ShapeContext::default(),
            &mut Textured {
                inner: renderer,
                texture: &self.texture,
            },
        );
    }
}
