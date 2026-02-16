//! Graphics rendering utilities for wgame.
//!
//! This crate provides a GPU rendering framework built on top of `wgpu`, offering
//! abstractions for rendering 2D and 3D content. It includes types for managing
//! rendering contexts, scenes, objects, and instances.
//!
//! # Core Concepts
//!
//! ## Renderer Architecture
//!
//! The rendering system is built around a few key traits:
//! - [`Renderer`] - A trait for objects that can render themselves given a context
//! - [`Context`] - A trait that provides binding groups for shader access
//! - [`Object`] - An object that can produce one or more instances for rendering
//! - [`Instance`] - A single renderable entity with a specific resource and storage
//!
//! ## Scene Management
//!
//! The [`Scene`] type collects all renderable objects and organizes them by
//! resource type. It automatically batches objects that share the same resource
//! for efficient rendering.
//!
//! ## Auto-Scenes
//!
//! The [`AutoScene`] is a convenience wrapper that automatically renders all
//! objects added to it when it's dropped. This simplifies the common case of
//! rendering a collection of objects in a single frame.
//!
//! # Usage
//!
//! ```
//! # use wgame_gfx::{Renderer, Context};
//! # struct MyContext;
//! # impl Context for MyContext { fn bind_group(&self) -> wgpu::BindGroup { unimplemented!() } }
//! # struct MyRenderer;
//! # impl<C: Context> Renderer<C> for MyRenderer {
//! #     fn render(&self, _ctx: &C, _pass: &mut wgpu::RenderPass<'_>) {}
//! # }
//! # fn example() -> anyhow::Result<()> {
//! // In a real application, you would:
//! // 1. Create a Surface with wgpu
//! // 2. Get a Frame from the Surface
//! // 3. Create a Camera (which is the default Context)
//! // 4. Add objects to the scene
//! // 5. Render using the frame
//! # Ok(())
//! # }
//! ```
//!
//! # Modules
//!
//! - [`auto`] - Auto-scene management with [`AutoScene`]
//! - [`camera`] - Camera context for rendering with [`Camera`]
//! - [`frame`] - Frame management with [`Frame`]
//! - [`instance`] - Instance and storage definitions
//! - [`modifiers`] - Transform and color modifiers
//! - [`object`] - Object trait and instance visitor
//! - [`order`] - Ordered rendering with [`Ordered`]
//! - [`renderer`] - Renderer and context traits
//! - [`resource`] - Resource definitions for batching
//! - [`scene`] - Scene management with [`Scene`]
//! - [`state`] - Graphics state with [`Graphics`]
//! - [`types`] - Basic types like [`Color`], [`Position`], [`Transform`]
//! - [`utils`] - Utility traits and functions

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
};
pub use anyhow::Error;
pub use wgpu::PresentMode;

/// Prelude module for easy imports.
///
/// This module re-exports commonly used types from the crate for convenience.
/// It includes:
/// - [`Object`] - For objects that can be rendered
/// - [`Renderer`] - For renderers that can draw objects
/// - All modifiers from [`modifiers`] module for transform and color operations
pub mod prelude {
    #[doc(no_inline)]
    pub use crate::{Object, Renderer, modifiers::*};
}

use anyhow::{Context as _, Result};

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

pub struct Surface<'a> {
    config: Config,
    surface: wgpu::Surface<'a>,
    state: Graphics,
    size: (u32, u32),
}

impl<'a> Surface<'a> {
    pub async fn new(
        config: Config,
        window_handle: impl Into<wgpu::SurfaceTarget<'a>>,
    ) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());

        let surface = instance
            .create_surface(window_handle)
            .context("Failed to create surface")?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .context("Failed to find an appropriate adapter")?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
                experimental_features: Default::default(),
            })
            .await
            .context("Failed to create device")?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];

        let this = Self {
            config,
            surface,
            state: Graphics::new(adapter, device, queue, format),
            size: Default::default(),
        };

        Ok(this)
    }

    fn configure(&self) {
        let size = self.size;
        if let (0, _) | (_, 0) = size {
            log::debug!("Invalid surface size: {size:?}, skipping configuration");
            return;
        }
        let surface_config = self
            .surface
            .get_default_config(self.state.adapter(), size.0, size.1)
            .unwrap();
        self.surface.configure(
            self.state.device(),
            &wgpu::SurfaceConfiguration {
                present_mode: self.config.present_mode,
                ..surface_config
            },
        );
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }
    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.size = new_size;
        self.configure();
    }

    pub fn frame(&mut self) -> Result<Frame<'a, '_>> {
        Frame::new(self)
    }

    pub fn state(&self) -> &Graphics {
        &self.state
    }
}
