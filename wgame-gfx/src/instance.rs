use crate::{Resource, modifiers::Ordered};

pub trait Context: Clone + Default {}

/// Single instance to draw.
pub trait Instance {
    type Resource: Resource;
    type Context: Context;

    fn resource(&self) -> Self::Resource;
    fn store(&self, context: &Self::Context, storage: &mut <Self::Resource as Resource>::Storage);
}

impl<T: Instance> Instance for &'_ T {
    type Resource = T::Resource;
    type Context = T::Context;

    fn resource(&self) -> Self::Resource {
        (*self).resource()
    }
    fn store(&self, params: &Self::Context, storage: &mut <Self::Resource as Resource>::Storage) {
        (*self).store(params, storage);
    }
}

pub trait InstanceExt: Instance + Sized {
    fn order(&self, order: i64) -> Ordered<&Self> {
        Ordered::new(self, order)
    }
}

impl<T: Instance> InstanceExt for T {}
