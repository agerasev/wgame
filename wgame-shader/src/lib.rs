//! Shader utilities for wgame graphics framework.
//!
//! This crate provides types and traits for working with GPU shaders, including:
//! - [`Attribute`] trait for defining shader input/output structures
//! - [`Binding`] and [`BindingList`] for managing shader resource bindings
//! - [`ShaderSource`] for templated shader code
//!
//! # Overview
//!
//! The crate is designed to bridge Rust types with GPU shader code (WGSL). It allows
//! you to define data structures in Rust that can be automatically serialized and
//! passed to shaders as uniform buffers or vertex attributes.
//!
//! # Attribute Trait
//!
//! The [`Attribute`] trait is the core of this crate. It defines how Rust types
//! map to GPU shader attributes:
//!
//! - [`Attribute::bindings`] - Returns the list of bindings for the shader
//! - [`Attribute::SIZE`] - Size of the data in bytes
//! - [`Attribute::store`] - Serializes the data into a byte sink
//!
//! # Deriving Attribute
//!
//! Use the [`wgame_shader_macros::Attribute`] derive macro to automatically
//! implement the `Attribute` trait for your structs:
//!
//! ```
//! # use wgame_shader_macros::Attribute;
//! #[derive(Attribute)]
//! struct Vertex {
//!     position: [f32; 3],
//!     color: [f32; 4],
//! }
//! ```
//!
//! # Built-in Implementations
//!
//! The crate provides `Attribute` implementations for:
//! - [`glam`] types: `Vec2`, `Vec3`, `Vec4`, `Mat2`, `Mat3`, `Mat4`, `Affine2`
//! - [`rgb`] types: `Rgba<f32>`, `Rgba<f16>`
//! - Primitive types: `f32`, `f16`, `u8`, `u16`, `u32`, `i8`, `i16`, `i32`
//!
//! # Example
//!
//! ```
//! use wgame_shader::{Attribute, BytesSink};
//! use wgame_shader_macros::Attribute;
//!
//! #[derive(Attribute)]
//! struct Uniforms {
//!     projection: glam::Mat4,
//!     view: glam::Mat4,
//! }
//!
//! let uniforms = Uniforms {
//!     projection: glam::Mat4::IDENTITY,
//!     view: glam::Mat4::IDENTITY,
//! };
//!
//! // Get the bindings for use in shader
//! let bindings = uniforms.bindings();
//!
//! // Serialize to bytes for GPU upload
//! let bytes = uniforms.to_bytes();
//! ```
//!
//! # Shader Templates
//!
//! The [`ShaderSource`] type allows you to use Jinja-style templates in your
//! shader code, with custom filters like `add` and `enumerate`:
//!
//! ```
//! # use wgame_shader::ShaderSource;
//! let source = ShaderSource::new("test", "uniforms: {{ n }}").unwrap();
//! let rendered = source.substitute(&serde_json::json!({"n": 42})).unwrap();
//! ```

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
