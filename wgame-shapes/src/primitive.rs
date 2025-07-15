use core::mem::offset_of;

use bytemuck::{Pod, Zeroable};
use glam::{Affine2, Mat4, Vec2, Vec4};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pos: [f32; 4],
    local_coord: [f32; 2],
}

impl Vertex {
    pub fn new(pos: Vec4, local_coord: Vec2) -> Self {
        Self {
            pos: pos.into(),
            local_coord: local_coord.into(),
        }
    }

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: offset_of!(Vertex, pos) as u64,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: offset_of!(Vertex, local_coord) as u64,
                    shader_location: 1,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Instance {
    xform_0: [f32; 4],
    xform_1: [f32; 4],
    xform_2: [f32; 4],
    xform_3: [f32; 4],
    tex_xform_0: [f32; 2],
    tex_xform_1: [f32; 2],
    tex_xform_2: [f32; 2],
}

impl Instance {
    pub fn new(xform: Mat4, tex_xform: Affine2) -> Self {
        let [xform_0, xform_1, xform_2, xform_3] = xform.to_cols_array_2d();
        let [tex_xform_0, tex_xform_1, tex_xform_2] = tex_xform.to_cols_array_2d();
        Self {
            xform_0,
            xform_1,
            xform_2,
            xform_3,
            tex_xform_0,
            tex_xform_1,
            tex_xform_2,
        }
    }

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: offset_of!(Instance, xform_0) as u64,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: offset_of!(Instance, xform_1) as u64,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: offset_of!(Instance, xform_2) as u64,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: offset_of!(Instance, xform_3) as u64,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: offset_of!(Instance, tex_xform_0) as u64,
                    shader_location: 6,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: offset_of!(Instance, tex_xform_1) as u64,
                    shader_location: 7,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: offset_of!(Instance, tex_xform_2) as u64,
                    shader_location: 8,
                },
            ],
        }
    }
}
