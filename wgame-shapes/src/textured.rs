use glam::{Affine2, Vec2};

use half::f16;
use rgb::{ComponentMap, Rgba};

use wgame_gfx::{Color, Context, Instance, Renderer, State, Texture, color};

use crate::{Shape, ShapeExt, bytes::StoreBytes, primitive::InstanceData, renderer::ShapeRenderer};

#[derive(Clone)]
pub struct Textured<'a, T: Shape<'a>> {
    shape: T,
    texture: Texture<'a>,
    color: Rgba<f16>,
}

impl<'a, T: Shape<'a>> Textured<'a, T> {
    pub fn new(shape: T, texture: Texture<'a>) -> Self {
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

impl<'a, T: Shape<'a>> Instance for Textured<'a, T> {
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

    fn store(&self, ctx: impl Context, storage: &mut <Self::Renderer as Renderer>::Storage) {
        InstanceData {
            xform: ctx.view_matrix() * self.shape.xform(),
            tex_xform: self.texture.coord_xform(),
            color: self.color.map(|x| x.to_f32()),
            custom: self.shape.attributes(),
        }
        .store_bytes(&mut storage.data);
        storage.count += 1;
    }
}

pub fn gradient<'a, T: Color, const N: usize>(state: &State<'a>, colors: [T; N]) -> Texture<'a> {
    gradient2(state, [colors])
}

pub fn gradient2<'a, T: Color, const M: usize, const N: usize>(
    state: &State<'a>,
    colors: [[T; M]; N],
) -> Texture<'a> {
    let colors = colors.map(|row| row.map(|color| color.to_rgba()));
    let pix_size = Vec2::new(M as f32, N as f32).recip();
    Texture::with_data(state, (M as u32, N as u32), colors.as_flattened()).transform_coord(
        Affine2::from_scale_angle_translation(1.0 - pix_size, 0.0, 0.5 * pix_size),
    )
}
