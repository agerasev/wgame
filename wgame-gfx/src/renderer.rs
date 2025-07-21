use core::{
    any::Any,
    hash::{Hash, Hasher},
};

use alloc::boxed::Box;
use anyhow::Result;

use crate::{Context, ContextExt, modifiers::Transformed, types::Transform, utils::AnyKey};

pub trait Renderer: Any + Eq + Hash {
    type Storage: Any;
    fn new_storage(&self) -> Self::Storage;
    fn draw(&self, instances: &Self::Storage, pass: &mut wgpu::RenderPass) -> Result<()>;
}

pub trait Instance {
    type Renderer: Renderer;
    fn get_renderer(&self) -> Self::Renderer;
    fn store(&self, ctx: impl Context, storage: &mut <Self::Renderer as Renderer>::Storage);
}

impl<T: Instance> Instance for &'_ T {
    type Renderer = T::Renderer;

    fn get_renderer(&self) -> Self::Renderer {
        T::get_renderer(self)
    }
    fn store(&self, ctx: impl Context, storage: &mut <Self::Renderer as Renderer>::Storage) {
        T::store(self, ctx, storage);
    }
}

pub trait InstanceExt: Instance + Sized {
    fn transform<T: Transform>(&self, xform: T) -> Transformed<&Self> {
        Transformed::new(self, xform)
    }
}

impl<T: Instance> InstanceExt for T {}

impl<T: Instance> Instance for Transformed<T> {
    type Renderer = T::Renderer;

    fn get_renderer(&self) -> Self::Renderer {
        self.inner.get_renderer()
    }
    fn store(&self, ctx: impl Context, storage: &mut <Self::Renderer as Renderer>::Storage) {
        self.inner.store(ctx.transform(self.xform), storage);
    }
}

pub trait AnyRenderer: AnyKey {
    fn new_dyn_storage(&self) -> Box<dyn Any>;
    fn draw_dyn(&self, instances: &dyn Any, pass: &mut wgpu::RenderPass) -> Result<()>;
}

impl<R: Renderer> AnyRenderer for R {
    fn new_dyn_storage(&self) -> Box<dyn Any> {
        Box::new(self.new_storage())
    }

    fn draw_dyn(&self, instances: &dyn Any, pass: &mut wgpu::RenderPass) -> Result<()> {
        let instances = instances
            .downcast_ref::<R::Storage>()
            .expect("Error downcasting storage during draw");
        self.draw(instances, pass)
    }
}

impl PartialEq for dyn AnyRenderer {
    fn eq(&self, other: &dyn AnyRenderer) -> bool {
        self.eq_dyn(other)
    }
}

impl Eq for dyn AnyRenderer {}

impl Hash for dyn AnyRenderer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_dyn(state);
    }
}

impl Renderer for dyn AnyRenderer {
    type Storage = Box<dyn Any>;

    fn new_storage(&self) -> Self::Storage {
        self.new_dyn_storage()
    }
    fn draw(&self, instances: &Self::Storage, pass: &mut wgpu::RenderPass) -> Result<()> {
        self.draw_dyn(&**instances, pass)
    }
}
