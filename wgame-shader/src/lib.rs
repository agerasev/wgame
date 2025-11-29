#![forbid(unsafe_code)]

mod attribute;
mod binding;
mod shader;

pub use self::{
    attribute::{Attribute, BytesSink},
    binding::{Binding, BindingList, BindingType},
    shader::ShaderSource,
};
pub use wgame_shader_macros::{Attribute, AttributeGlobal};
