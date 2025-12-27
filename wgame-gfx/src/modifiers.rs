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
    fn mul_color<C: Color>(&self, color: C) -> Self;
}
