use std::hash::Hash;

use anyhow::Result;

use crate::{Context, Instance, Renderer, Resource, utils::AnyOrder};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Ordered<T> {
    pub inner: T,
    pub order: i64,
}

impl<T> Ordered<T> {
    pub fn new(inner: T, order: i64) -> Self {
        Ordered { inner, order }
    }
}

impl<T: Instance> Instance for Ordered<T> {
    type Resource = Ordered<T::Resource>;

    fn resource(&self) -> Self::Resource {
        Ordered::new(self.inner.resource(), self.order)
    }
    fn store(&self, ctx: &Context, storage: &mut <Self::Resource as Resource>::Storage) {
        self.inner.store(ctx, storage);
    }
}

impl<T: Resource> Resource for Ordered<T> {
    type Renderer = Ordered<T::Renderer>;
    type Storage = T::Storage;

    fn new_storage(&self) -> Self::Storage {
        self.inner.new_storage()
    }
    fn make_renderer(&self, instances: &Self::Storage) -> Result<Self::Renderer> {
        Ok(Ordered::new(
            self.inner.make_renderer(instances)?,
            self.order,
        ))
    }
}

impl<T: Renderer + Ord + Hash> Renderer for Ordered<T> {
    fn draw(&self, pass: &mut wgpu::RenderPass<'_>) -> Result<()> {
        self.inner.draw(pass)
    }
}

impl<T: Renderer> AnyOrder for Ordered<T> {
    fn order(&self) -> i64 {
        self.order
    }
}
