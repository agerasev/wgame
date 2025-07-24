#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

pub mod attributes;
pub mod binding;
pub mod bytes;
mod circle;
mod instance;
mod pipeline;
mod polygon;
pub mod primitive;
mod renderer;
mod shader;
mod shape;
mod texture;

use alloc::rc::Rc;
use core::ops::Deref;

use anyhow::Result;

use wgame_gfx::{
    Graphics,
    types::{Color, color},
};

use crate::{circle::CircleLibrary, polygon::PolygonLibrary};

pub use self::{
    instance::Textured,
    polygon::Polygon,
    shape::{Shape, ShapeExt},
    texture::Texture,
};

/// Library shared state
pub struct InnerState {
    inner: Graphics,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_sampler: wgpu::Sampler,
}

pub type LibraryState = Rc<InnerState>;

impl Deref for InnerState {
    type Target = Graphics;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl InnerState {
    fn new(state: &Graphics) -> Self {
        Self {
            inner: state.clone(),
            texture_bind_group_layout: Texture::create_bind_group_layout(state),
            texture_sampler: Texture::create_sampler(state),
        }
    }
}

/// 2D graphics library
#[derive(Clone)]
pub struct Library {
    state: LibraryState,
    polygon: PolygonLibrary,
    circle: CircleLibrary,
    white_texture: Texture,
}

impl Deref for Library {
    type Target = LibraryState;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl Library {
    pub fn new(state: &Graphics) -> Result<Self> {
        let state = Rc::new(InnerState::new(state));
        Ok(Self {
            polygon: PolygonLibrary::new(&state)?,
            circle: CircleLibrary::new(&state)?,
            white_texture: {
                let tex = Texture::new(&state, (1, 1));
                tex.write(&[color::WHITE.to_rgba()]);
                tex
            },
            state,
        })
    }
}
