//! GPU rendering framework built on WGPU.
//!
//! Provides abstractions for rendering 2D content with scene management, batching, and camera support.

#![forbid(unsafe_code)]

mod auto;
mod camera;
mod frame;
mod instance;
pub mod modifiers;
mod object;
mod order;
mod renderer;
mod resource;
mod scene;
mod state;
mod surface;
mod target;
pub mod types;
pub mod utils;

pub use self::{
    auto::AutoScene,
    camera::Camera,
    frame::Frame,
    instance::{AnyStorage, Instance, Storage},
    object::{InstanceVisitor, Object},
    order::Ordered,
    renderer::{Context, Renderer},
    resource::{AnyResource, Resource},
    scene::Scene,
    state::Graphics,
    surface::Surface,
    target::Target,
};
pub use anyhow::Error;
pub use wgpu::PresentMode;

/// Commonly used types and traits.
pub mod prelude {
    #[doc(no_inline)]
    pub use crate::{Object, Renderer, Target, modifiers::*};
}

#[derive(Clone, Debug)]
pub struct Config {
    pub present_mode: wgpu::PresentMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            present_mode: wgpu::PresentMode::AutoVsync,
        }
    }
}
