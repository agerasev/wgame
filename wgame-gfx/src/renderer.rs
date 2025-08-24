use alloc::boxed::Box;
use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use anyhow::Result;

use crate::{
    Context, Instance, Resources,
    utils::{AnyKey, AnyOrder},
};

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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RendererInstance<R: Renderer + Clone + Ord + Hash>(pub R);
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RendererResources<R: Renderer + Clone + Ord + Hash>(pub R);

impl<T: Renderer + Clone + Ord + Hash> Instance for RendererInstance<T> {
    type Resources = RendererResources<T>;

    fn get_resources(&self) -> Self::Resources {
        RendererResources(self.0.clone())
    }
    fn store(&self, _ctx: &Context, _storage: &mut <Self::Resources as Resources>::Storage) {}
}

impl<T: Renderer + Clone + Ord + Hash> Resources for RendererResources<T> {
    type Renderer = T;
    type Storage = ();

    fn new_storage(&self) -> Self::Storage {}
    fn make_renderer(&self, _instances: &Self::Storage) -> Result<Self::Renderer> {
        Ok(self.0.clone())
    }
}
