use glam::{Affine2, Mat4, Vec2, Vec4};

use wgame_macros::{Attributes, StoreBytes};

use crate as wgame_shapes;

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
pub struct Instance {
    xform: Mat4,
    tex_xform: Affine2,
}

impl Instance {
    pub fn new(xform: Mat4, tex_xform: Affine2) -> Self {
        Self { xform, tex_xform }
    }
}
