#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;

mod atlas;
mod texture;

use alloc::rc::Rc;
use core::ops::Deref;

use wgame_gfx::Graphics;

pub use self::texture::Texture;

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
    pub fn new(state: &Graphics) -> Self {
        Self {
            inner: state.clone(),
            texture_bind_group_layout: Texture::create_bind_group_layout(state),
            texture_sampler: Texture::create_sampler(state),
        }
    }
}
