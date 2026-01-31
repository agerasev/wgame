use glam::Affine3A;
use wgame_gfx::{modifiers::Transformable, types::Color};
use wgame_shader::Attribute;

use crate::{Mesh, ShapesLibrary, ShapesState, Texture, Textured};

pub trait ElementVisitor {
    fn visit<T: Element>(&mut self, element: &T);
}

pub trait Element: Clone {
    type Attribute: Attribute;

    fn state(&self) -> &ShapesState;
    fn vertices(&self) -> Mesh;
    fn uniforms(&self) -> Option<wgpu::BindGroup> {
        None
    }
    fn attribute(&self) -> Self::Attribute;
    fn pipeline(&self) -> wgpu::RenderPipeline;
    fn xform(&self) -> Affine3A;
}

pub trait Shape: Transformable + Clone {
    fn library(&self) -> &ShapesLibrary;
    fn for_each_element<V: ElementVisitor>(&self, visitor: &mut V);
}

pub trait ShapeExt: Shape + Clone {
    fn with_texture(&self, texture: impl AsRef<Texture>) -> Textured<Self> {
        Textured::new(self.clone(), texture.as_ref().clone())
    }
    fn with_color(&self, color: impl Color) -> Textured<Self> {
        Textured {
            inner: self.clone(),
            texture: self.library().white_texture.multiply_color(color),
        }
    }
}

impl<T: Shape> ShapeExt for T {}
