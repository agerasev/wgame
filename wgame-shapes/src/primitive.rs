use glam::{Affine2, Mat4, Vec2, Vec4};

use wgame_macros::{Attributes, StoreBytes};

use crate::{self as wgame_shapes, attributes::Attributes};

#[derive(Clone, Copy, StoreBytes, Attributes)]
pub struct Vertex {
    pub pos: Vec4,
    pub local_coord: Vec2,
}

impl Vertex {
    pub fn new(pos: Vec4, local_coord: Vec2) -> Self {
        Self { pos, local_coord }
    }
}

#[derive(Clone, Copy, StoreBytes, Attributes)]
pub struct Instance<T: Attributes = ()> {
    xform: Mat4,
    tex_xform: Affine2,
    custom: T,
}

impl<T: Attributes> Instance<T> {
    pub fn new(xform: Mat4, tex_xform: Affine2, custom: T) -> Self {
        Self {
            xform,
            tex_xform,
            custom,
        }
    }
}
