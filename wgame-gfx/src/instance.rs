use crate::Resource;

/// Single instance to draw.
pub trait Instance {
    type Resource: Resource;

    fn resource(&self) -> Self::Resource;
    fn store(&self, storage: &mut <Self::Resource as Resource>::Storage);
}

impl<T: Instance> Instance for &'_ T {
    type Resource = T::Resource;

    fn resource(&self) -> Self::Resource {
        (*self).resource()
    }
    fn store(&self, storage: &mut <Self::Resource as Resource>::Storage) {
        (*self).store(storage);
    }
}
