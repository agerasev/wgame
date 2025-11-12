use std::{
    any::Any,
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use anyhow::Result;

use crate::{
    Context,
    modifiers::{Colored, Ordered, Transformed},
    renderer::Renderer,
    types::{Color, Transform},
    utils::AnyKey,
};

/// Shared resource required to draw an instance.
///
/// Equality of the instances' resource means that they can be draw in single render pass.
pub trait Resource: Any + Ord + Hash {
    type Storage: Any;
    type Renderer: Renderer + Ord + Hash;

    fn new_storage(&self) -> Self::Storage;
    fn make_renderer(&self, instances: &Self::Storage) -> Result<Self::Renderer>;
}

/// Single instance to draw.
pub trait Instance {
    type Resource: Resource;
    fn resource(&self) -> Self::Resource;
    fn store(&self, ctx: &Context, storage: &mut <Self::Resource as Resource>::Storage);
}

impl<T: Instance> Instance for &'_ T {
    type Resource = T::Resource;

    fn resource(&self) -> Self::Resource {
        T::resource(self)
    }
    fn store(&self, ctx: &Context, storage: &mut <Self::Resource as Resource>::Storage) {
        T::store(self, ctx, storage);
    }
}

pub trait InstanceExt: Instance + Sized {
    fn transform<T: Transform>(&self, xform: T) -> Transformed<&Self> {
        Transformed::new(self, xform)
    }
    fn color<C: Color>(&self, color: C) -> Colored<&Self> {
        Colored::new(self, color)
    }
    fn order(&self, order: i64) -> Ordered<&Self> {
        Ordered::new(self, order)
    }
}
impl<T: Instance> InstanceExt for T {}

pub trait AnyResource: AnyKey {
    fn new_dyn_storage(&self) -> Box<dyn Any>;
    fn make_dyn_renderer(&self, instances: &dyn Any) -> Result<Box<dyn Renderer>>;
}

impl<R: Resource> AnyResource for R {
    fn new_dyn_storage(&self) -> Box<dyn Any> {
        Box::new(self.new_storage())
    }

    fn make_dyn_renderer(&self, instances: &dyn Any) -> Result<Box<dyn Renderer>> {
        let instances = instances
            .downcast_ref::<R::Storage>()
            .expect("Error downcasting storage during draw");
        Ok(Box::new(self.make_renderer(instances)?))
    }
}

impl PartialEq for dyn AnyResource {
    fn eq(&self, other: &dyn AnyResource) -> bool {
        self.eq_dyn(other)
    }
}
impl Eq for dyn AnyResource {}
impl PartialOrd for dyn AnyResource {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for dyn AnyResource {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_dyn(other)
    }
}
impl Hash for dyn AnyResource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_dyn(state);
    }
}

impl Resource for dyn AnyResource {
    type Storage = Box<dyn Any>;
    type Renderer = Box<dyn Renderer>;

    fn new_storage(&self) -> Self::Storage {
        self.new_dyn_storage()
    }
    fn make_renderer(&self, instances: &Self::Storage) -> Result<Self::Renderer> {
        self.make_dyn_renderer(&**instances)
    }
}

impl Resource for Box<dyn AnyResource> {
    type Storage = <dyn AnyResource as Resource>::Storage;
    type Renderer = <dyn AnyResource as Resource>::Renderer;

    fn new_storage(&self) -> Self::Storage {
        (**self).new_dyn_storage()
    }
    fn make_renderer(&self, instances: &Self::Storage) -> Result<Self::Renderer> {
        (**self).make_dyn_renderer(&**instances)
    }
}
