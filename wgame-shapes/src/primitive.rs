use glam::{Mat4, Vec2, Vec4};

use rgb::Rgba;
use wgame_macros::{Attributes, StoreBytes};
use wgame_texture::Texture;

use crate::attributes::Attributes;

#[derive(Clone, Copy, StoreBytes, Attributes)]
#[bytes_mod(wgame_gfx::bytes)]
#[attributes_mod(crate::attributes)]
pub struct VertexData {
    pub pos: Vec4,
    pub local_coord: Vec2,
}

impl VertexData {
    pub fn new(pos: Vec4, local_coord: Vec2) -> Self {
        Self { pos, local_coord }
    }
}

#[derive(Clone, StoreBytes, Attributes)]
#[bytes_mod(wgame_gfx::bytes)]
#[attributes_mod(crate::attributes)]
pub struct InstanceData<T: Attributes = ()> {
    pub xform: Mat4,
    pub tex_xform: Texture,
    pub color: Rgba<f32>,
    pub custom: T,
}
