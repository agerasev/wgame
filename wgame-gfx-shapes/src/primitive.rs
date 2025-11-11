use glam::{Mat4, Vec2, Vec4};
use rgb::Rgba;
use wgame_gfx_texture::TextureAttribute;
use wgame_shader::Attribute;

#[derive(Clone, Copy, Attribute)]
pub struct VertexData {
    pub pos: Vec4,
    pub local_coord: Vec2,
}

impl VertexData {
    pub fn new(pos: Vec4, local_coord: Vec2) -> Self {
        Self { pos, local_coord }
    }
}

#[derive(Clone, Attribute)]
pub struct InstanceData<T: Attribute = ()> {
    pub xform: Mat4,
    pub tex_xform: TextureAttribute,
    pub color: Rgba<f32>,
    pub custom: T,
}
