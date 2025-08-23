use alloc::boxed::Box;
use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use anyhow::Result;

use crate::utils::{AnyKey, AnyOrder};

pub trait Renderer: AnyKey + AnyOrder {
    fn draw(&self, pass: &mut wgpu::RenderPass<'_>) -> Result<()>;
}

impl PartialEq for dyn Renderer {
    fn eq(&self, other: &dyn Renderer) -> bool {
        self.eq_dyn(other)
    }
}
impl Eq for dyn Renderer {}

impl PartialOrd for dyn Renderer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for dyn Renderer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_dyn(other)
    }
}

impl Hash for dyn Renderer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_dyn(state);
    }
}

impl Renderer for Box<dyn Renderer> {
    fn draw(&self, pass: &mut wgpu::RenderPass<'_>) -> Result<()> {
        (**self).draw(pass)
    }
}
