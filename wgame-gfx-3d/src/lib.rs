//! 3D primitives and lighting using the shared mesh renderer.
//!
//! Geometry, materials, scenes, cameras, textures, transforms and depth remain
//! shared with 2D. This crate supplies surface-lighting equations and 3D geometry.
//! [`Lighting`] adds ambient and directional diffuse/specular lighting, with
//! optional tangent-space normal maps. Unlit shapes can share the same scene.
#![forbid(unsafe_code)]
mod lighting;
mod normals;
mod primitives;
pub use lighting::*;
pub use normals::*;
pub use primitives::{Shapes3d, cylinder_mesh};
