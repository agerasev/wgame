use glam::{Mat4, Vec3, Vec4};
use serde::Serialize;
use wgame_gfx_texture::TextureAttribute;
use wgame_shader::{Attribute, Binding, BindingList};

#[derive(Clone, Default, Debug, Serialize)]
pub struct ShaderConfig {
    /// Instance buffer additional attributes.
    pub instance: BindingList,
    /// Additional variables to pass from vertex shader to fragment shader.
    pub varying: BindingList,
    /// Uniforms to pass to fragment shader.
    pub fragment_uniforms: Vec<Binding>,

    /// Code to execute in vertex shader to set additional varyings and alter basic ones.
    pub vertex_source: String,
    /// Code to execute in fragment shader before texture sampling.
    pub fragment_texcoord_source: String,
    /// Code to execute in fragment shader after texture sampling.
    /// Discarding should be done here if needed.
    pub fragment_color_source: String,
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
    pub matrix: Mat4,
    pub tex: TextureAttribute,
    pub custom: T,
}
