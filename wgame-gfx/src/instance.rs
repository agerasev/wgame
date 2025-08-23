use core::{
    any::Any,
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use alloc::boxed::Box;
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
/// Equality of the instances' resources means that they can be draw in single render pass.
pub trait Resources: Any + Ord + Hash {
    type Storage: Any;
    type Renderer: Renderer + Ord + Hash;

    fn new_storage(&self) -> Self::Storage;
    fn make_renderer(&self, instances: &Self::Storage) -> Result<Self::Renderer>;
}

/// Single instance to draw.
pub trait Instance {
    type Resources: Resources;
    fn get_resources(&self) -> Self::Resources;
    fn store(&self, ctx: &Context, storage: &mut <Self::Resources as Resources>::Storage);
}

impl<T: Instance> Instance for &'_ T {
    type Resources = T::Resources;

    fn get_resources(&self) -> Self::Resources {
        T::get_resources(self)
    }
    fn store(&self, ctx: &Context, storage: &mut <Self::Resources as Resources>::Storage) {
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

pub trait AnyResources: AnyKey {
    fn new_dyn_storage(&self) -> Box<dyn Any>;
    fn make_dyn_renderer(&self, instances: &dyn Any) -> Result<Box<dyn Renderer>>;
}

impl<R: Resources> AnyResources for R {
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

impl PartialEq for dyn AnyResources {
    fn eq(&self, other: &dyn AnyResources) -> bool {
        self.eq_dyn(other)
    }
}
impl Eq for dyn AnyResources {}
impl PartialOrd for dyn AnyResources {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for dyn AnyResources {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_dyn(other)
    }
}
impl Hash for dyn AnyResources {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_dyn(state);
    }
}

impl Resources for dyn AnyResources {
    type Storage = Box<dyn Any>;
    type Renderer = Box<dyn Renderer>;

    fn new_storage(&self) -> Self::Storage {
        self.new_dyn_storage()
    }
    fn make_renderer(&self, instances: &Self::Storage) -> Result<Self::Renderer> {
        self.make_dyn_renderer(&**instances)
    }
}

impl Resources for Box<dyn AnyResources> {
    type Storage = <dyn AnyResources as Resources>::Storage;
    type Renderer = <dyn AnyResources as Resources>::Renderer;

    fn new_storage(&self) -> Self::Storage {
        (**self).new_dyn_storage()
    }
    fn make_renderer(&self, instances: &Self::Storage) -> Result<Self::Renderer> {
        (**self).make_dyn_renderer(&**instances)
    }
}
