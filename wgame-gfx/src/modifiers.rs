use crate::types::{Color, Transform};

pub trait Transformable: Sized {
    fn transform<X: Transform>(&self, xform: X) -> Self;
}

pub trait Colorable: Sized {
    fn multiply_color<C: Color>(&self, color: C) -> Self;
}
