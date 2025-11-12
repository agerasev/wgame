use crate::{Context, Resource, modifiers::Ordered};

/// Single instance to draw.
pub trait Instance {
    type Resource: Resource;
    fn resource(&self) -> Self::Resource;
    fn store(&self, ctx: &Context, storage: &mut <Self::Resource as Resource>::Storage);
}

pub trait InstanceExt: Instance + Sized {
    fn order(&self, order: i64) -> Ordered<&Self> {
        Ordered::new(self, order)
    }
}

impl<T: Instance> InstanceExt for T {}
