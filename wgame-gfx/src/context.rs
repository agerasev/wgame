use glam::Mat4;

use crate::{Transformed, types::Transform};

pub trait Context {
    fn view_matrix(&self) -> Mat4;
}

#[derive(Clone, Default, Debug)]
pub struct DefaultContext;

impl Context for DefaultContext {
    fn view_matrix(&self) -> Mat4 {
        Mat4::IDENTITY
    }
}

impl<C: Context> Context for &C {
    fn view_matrix(&self) -> Mat4 {
        C::view_matrix(self)
    }
}

pub trait ContextExt: Context {
    fn transform<T: Transform>(&self, xform: T) -> Transformed<&Self> {
        Transformed::new(self, xform)
    }
}

impl<C: Context> ContextExt for C {}

impl<C: Context> Context for Transformed<C> {
    fn view_matrix(&self) -> Mat4 {
        self.inner.view_matrix() * self.xform
    }
}
