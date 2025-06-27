pub mod triangle;

use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec4};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub pos: Vec4,
    pub tex_coord: Vec2,
    pub _pad: Vec2,
}
