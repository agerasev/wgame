//! Shader utilities for bridging Rust types with GPU shaders.
//!
//! Provides attribute traits, binding definitions, and shader templates for WGSL.

#![forbid(unsafe_code)]

mod attribute;
mod binding;
mod shader;

pub use self::{
    attribute::{Attribute, BytesSink},
    binding::{Binding, BindingList, BindingType, ScalarType},
    shader::ShaderSource,
};
pub use wgame_shader_macros::{Attribute, AttributeGlobal};
