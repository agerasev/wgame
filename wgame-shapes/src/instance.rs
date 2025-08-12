use half::f16;
use rgb::{ComponentMap, Rgba};

use wgame_gfx::{
    Context, Instance, Renderer,
    types::{Color, color},
};

use crate::{Shape, Texture, bytes::StoreBytes, primitive::InstanceData, renderer::ShapeRenderer};

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
    type Renderer = ShapeRenderer;

    fn get_renderer(&self) -> Self::Renderer {
        ShapeRenderer {
            order: 0,
            vertices: self.shape.vertices(),
            uniforms: [self.texture.bind_group().clone()]
                .into_iter()
                .chain(self.shape.uniforms())
                .collect(),
            pipeline: self.shape.pipeline(),
            device: self.shape.state().device().clone(),
        }
    }

    fn store(&self, ctx: &Context, storage: &mut <Self::Renderer as Renderer>::Storage) {
        InstanceData {
            xform: ctx.view * self.shape.xform(),
            tex_xform: self.texture.coord_xform(),
            color: self.color.map(|x| x.to_f32()),
            custom: self.shape.attributes(),
        }
        .store_bytes(&mut storage.data);
        storage.count += 1;
    }
}
