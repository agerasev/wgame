use glam::{Mat4, Vec3, Vec4};
use rgb::Rgba;
use serde::Serialize;
use wgame_gfx_texture::TextureAttribute;
use wgame_shader::{Attribute, Binding, BindingList};

#[derive(Clone, Default, Debug, Serialize)]
pub struct ShaderConfig {
    pub instances: BindingList,
    pub uniforms: Vec<Binding>,
    pub vertex_modifier: String,
    pub fragment_modifier: String,
}

#[derive(Clone, Copy, Attribute)]
pub struct VertexData {
    pub pos: Vec4,
    pub local_coord: Vec3,
}

impl VertexData {
    pub fn new(pos: Vec4, local_coord: Vec3) -> Self {
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
