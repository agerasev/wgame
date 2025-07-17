use glam::Mat4;

use wgame_gfx::{
    Model, Object, Texture,
    bytes::{BytesSink, StoreBytes},
};

use crate::{Shape, primitive::Instance};

pub struct Textured<'a, T: Shape<'a>> {
    shape: T,
    texture: Texture<'a>,
}

impl<'a, T: Shape<'a>> Textured<'a, T> {
    pub fn new(shape: T, texture: Texture<'a>) -> Self {
        Self { shape, texture }
    }
}

impl<'a, T: Shape<'a>> Object for Textured<'a, T> {
    fn model(&self) -> Model {
        Model {
            index: 0,
            vertices: self.shape.vertices(),
            uniforms: [self.texture.bind_group().clone()]
                .into_iter()
                .chain(self.shape.uniforms())
                .collect(),
            pipeline: self.shape.pipeline(),
        }
    }

    fn store_instance<D: BytesSink>(&self, xform: Mat4, buffer: &mut D) {
        Instance::new(
            xform * self.shape.xform(),
            self.texture.coord_xform(),
            self.shape.attributes(),
        )
        .store_bytes(buffer);
    }
}
