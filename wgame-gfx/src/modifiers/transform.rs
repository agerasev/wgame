use glam::Mat4;

use crate::{Camera, Object, types::Transform};

#[derive(Clone, Debug)]
pub struct Transformed<T> {
    pub inner: T,
    pub xform: Mat4,
}

impl<T> Transformed<T> {
    pub fn new<X: Transform>(inner: T, xform: X) -> Self {
        Transformed {
            inner,
            xform: xform.to_mat4(),
        }
    }
}

impl<T: Object> Object for Transformed<T> {
    fn collect_into(&self, camera: &Camera, collector: &mut crate::Collector) {
        self.inner
            .collect_into(&camera.transform(self.xform), collector);
    }
}
