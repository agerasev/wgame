use glam::Mat4;

use crate::{Camera, Object, object::InstanceVisitor, types::Transform};

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
    fn visit_instances<V: InstanceVisitor>(&self, camera: &Camera, visitor: &mut V) {
        self.inner
            .visit_instances(&camera.transform(self.xform), visitor);
    }
}
