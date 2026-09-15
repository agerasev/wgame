//! GPU rendering with ordered scenes, reusable renderers, and window/offscreen targets.
//!
//! Collect changing objects in a [`Scene`], then draw through [`Target::render_iter`].
//! For unchanged content, [`Scene::bake`] produces a [`BakedScene`] that reuses its
//! instance buffers across frames and cameras. See those types for painter order,
//! resource lifetimes, and rebaking rules.
//!
//! [`Target::camera`] supplies aspect-correct coordinates; [`Target::physical_camera`]
//! uses physical pixels. [`AutoScene`] provides automatic rendering on normal drop.
//!
//! # Custom and headless rendering
//!
//! Use [`Graphics::device`] and [`Graphics::queue`] for direct wgpu access. Implement
//! [`Renderer<C>`] to encode drawing in a supplied render pass and [`Context`] for
//! shared bindings. [`Target`] abstracts window frames and [`Offscreen`] textures.
//! [`Graphics::new`] wraps a supplied adapter/device/queue for headless applications.
//! The facade's `raw_wgpu` example demonstrates a custom pipeline.

#![forbid(unsafe_code)]

mod auto;
mod camera;
mod depth;
mod frame;
mod instance;
pub mod modifiers;
mod object;
mod offscreen;
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
    depth::{DEPTH_FORMAT, DepthBuffer, DepthMode},
    frame::Frame,
    instance::{AnyStorage, Instance, Storage},
    object::{InstanceVisitor, Object},
    offscreen::Offscreen,
    order::Ordered,
    renderer::{Context, Renderer},
    resource::{AnyResource, Resource},
    scene::{BakedScene, Scene},
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
