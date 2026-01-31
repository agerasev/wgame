use glam::{Affine3A, Vec3};

use crate::types::{Color, Position, Transform};

pub trait Transformable: Sized {
    fn transform<X: Transform>(&self, xform: X) -> Self;

    fn scale(&self, factor: f32) -> Self {
        self.transform(Affine3A::from_scale(Vec3::splat(factor)))
    }
    fn move_to<P: Position>(&self, pos: P) -> Self {
        self.transform(Affine3A::from_translation(pos.to_xyz()))
    }
}

pub trait Colorable: Sized {
    fn multiply_color<C: Color>(&self, color: C) -> Self;
}

#[macro_export]
macro_rules! impl_transformable {
    ($self:ty, $field:ident) => {
        impl $crate::modifiers::Transformable for $self {
            fn transform<X: $crate::types::Transform>(&self, xform: X) -> Self {
                Self {
                    $field: xform.to_affine3() * self.$field,
                    ..self.clone()
                }
            }
        }
    };
}

#[macro_export]
macro_rules! delegate_transformable {
    ($self:ty, $inner:ident) => {
        impl $crate::modifiers::Transformable for $self {
            fn transform<X: $crate::types::Transform>(&self, xform: X) -> Self {
                Self {
                    $inner: $crate::modifiers::Transformable::transform(&self.$inner, xform),
                    ..self.clone()
                }
            }
        }
    };
}
