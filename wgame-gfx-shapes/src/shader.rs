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

    /// Module-level WGSL declarations and helper functions.
    pub module_source: String,

    /// Code to execute in vertex shader to set additional varyings and alter basic ones.
    pub vertex_source: String,
    /// Code to execute in fragment shader before texture sampling.
    pub fragment_texcoord_source: String,
    /// Code to execute in fragment shader after texture sampling.
    /// Discarding should be done here if needed.
    pub fragment_color_source: String,
}

#[derive(Clone, Copy, Attribute)]
pub struct Vertex {
    pub pos: Vec4,
    pub local_coord: Vec3,
    pub color: Vec4,
    /// Object-space surface normal. Flat 2D geometry uses +Z.
    pub normal: Vec3,
}

impl Vertex {
    /// Per-vertex tint multiplied by the texture and instance color.
    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }
    /// Set the object-space normal used by lighting materials.
    pub fn with_normal(mut self, normal: Vec3) -> Self {
        self.normal = normal;
        self
    }
    pub fn new(pos: Vec4, local_coord: Vec3) -> Self {
        Self {
            pos,
            local_coord,
            color: Vec4::ONE,
            normal: Vec3::Z,
        }
    }
}

#[derive(Clone, Attribute)]
pub struct InstanceData<T: Attribute = ()> {
    pub matrix: Mat4,
    pub tex: TextureAttribute,
    pub custom: T,
}
