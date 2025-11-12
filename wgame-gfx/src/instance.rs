use crate::{Camera, Resource, modifiers::Ordered};

/// Single instance to draw.
pub trait Instance {
    type Resource: Resource;
    fn resource(&self) -> Self::Resource;
    fn store(&self, camera: &Camera, storage: &mut <Self::Resource as Resource>::Storage);
}

impl<T: Instance> Instance for &'_ T {
    type Resource = T::Resource;

    fn resource(&self) -> Self::Resource {
        (*self).resource()
    }

    fn store(&self, camera: &Camera, storage: &mut <Self::Resource as Resource>::Storage) {
        (*self).store(camera, storage);
    }
}

pub trait InstanceExt: Instance + Sized {
    fn order(&self, order: i64) -> Ordered<&Self> {
        Ordered::new(self, order)
    }
}

impl<T: Instance> InstanceExt for T {}
