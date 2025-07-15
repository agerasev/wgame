use alloc::vec;

use glam::Mat4;

use wgame_gfx::{Model, Object, Texture};

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
            uniforms: vec![self.texture.bind_group().clone()],
            pipeline: self.shape.pipeline(),
        }
    }
    fn store_instance<D: Extend<u8>>(&self, xform: Mat4, buffer: &mut D) {
        buffer.extend(
            bytemuck::cast_slice(&[Instance::new(
                xform * self.shape.xform(),
                self.texture.coord_xform(),
            )])
            .iter()
            .copied(),
        )
    }
}
