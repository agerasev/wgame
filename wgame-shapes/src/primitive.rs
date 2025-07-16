use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::mem::replace;

use anyhow::Result;
use glam::{Affine2, Mat4, Vec2, Vec4};

use wgame_gfx::BytesSink;

use crate::{binding::BindingType, binding_type};

pub trait StoreBytes {
    const SIZE: usize;
    fn store_bytes<D: BytesSink>(&self, dst: &mut D);

    fn to_bytes(&self) -> Vec<u8> {
        let mut dst = Vec::new();
        self.store_bytes(&mut dst);
        dst
    }
}

impl<T: StoreBytes, const N: usize> StoreBytes for [T; N] {
    const SIZE: usize = T::SIZE * N;

    fn store_bytes<D: BytesSink>(&self, dst: &mut D) {
        for item in self {
            item.store_bytes(dst);
        }
    }
}

#[derive(Clone, Debug)]
pub struct AttributeInfo {
    name: String,
    ty: BindingType,
}

#[derive(Clone, Debug)]
pub struct AttributeList(pub Vec<AttributeInfo>);

impl AttributeList {
    pub fn chain(mut self, other: Self) -> Self {
        self.0.extend(other.0);
        Self(self.0)
    }

    pub fn with_prefix(mut self, prefix: &str) -> Self {
        for AttributeInfo { name, .. } in self.0.iter_mut() {
            *name = if name.is_empty() {
                prefix.to_string()
            } else {
                format!("{prefix}_{name}")
            };
        }
        self
    }

    pub fn size(&self) -> u64 {
        self.0
            .iter()
            .map(|AttributeInfo { ty, .. }| ty.size())
            .sum()
    }

    pub fn count(&self) -> u32 {
        self.0.len() as u32
    }

    pub fn layout(&self, start_location: u32) -> Result<Vec<wgpu::VertexAttribute>> {
        self.0
            .iter()
            .scan(
                (start_location, 0),
                |(index, offset), AttributeInfo { name, ty }| {
                    Some(Ok(wgpu::VertexAttribute {
                        shader_location: replace(index, *index + 1),
                        offset: replace(offset, *offset + ty.size()),
                        format: match ty.to_attribute() {
                            Ok(a) => a,
                            Err(e) => {
                                return Some(Err(e.context(format!(
                                    "Error getting attribute '{name}' of type {ty:?}",
                                ))));
                            }
                        },
                    }))
                },
            )
            .collect()
    }
}

pub trait Attributes {
    fn attributes() -> AttributeList;
}

macro_rules! impl_store_for_pod {
    ($type:ty, $layout:expr) => {
        impl StoreBytes for $type {
            const SIZE: usize = size_of::<$type>();

            fn store_bytes<D: BytesSink>(&self, dst: &mut D) {
                dst.push_bytes(bytemuck::bytes_of(self));
            }
        }

        impl Attributes for $type {
            fn attributes() -> AttributeList {
                let mut attrs: Vec<_> = $layout
                    .into_iter()
                    .map(|ty| AttributeInfo {
                        name: String::new(),
                        ty,
                    })
                    .collect();
                if attrs.len() > 1 {
                    for (i, attr) in attrs.iter_mut().enumerate() {
                        attr.name = format!("{i}");
                    }
                }
                AttributeList(attrs)
            }
        }
    };
}

impl_store_for_pod!(glam::Mat4, (0..4).map(|_| binding_type!(F32, 4)));
impl_store_for_pod!(glam::Mat3, (0..3).map(|_| binding_type!(F32, 4)));
impl_store_for_pod!(glam::Mat2, [binding_type!(F32, 4)]);
impl_store_for_pod!(glam::Vec4, [binding_type!(F32, 4)]);
impl_store_for_pod!(glam::Vec3, [binding_type!(F32, 3)]);
impl_store_for_pod!(glam::Vec2, [binding_type!(F32, 2)]);
impl_store_for_pod!(f32, [binding_type!(F32)]);

impl StoreBytes for glam::Affine2 {
    const SIZE: usize = size_of::<glam::Mat2>() + size_of::<glam::Vec2>();

    fn store_bytes<D: BytesSink>(&self, dst: &mut D) {
        dst.push_bytes(bytemuck::bytes_of(&self.matrix2));
        dst.push_bytes(bytemuck::bytes_of(&self.translation));
    }
}

impl Attributes for Affine2 {
    fn attributes() -> AttributeList {
        AttributeList(vec![
            AttributeInfo {
                name: "m".into(),
                ty: binding_type!(F32, 4),
            },
            AttributeInfo {
                name: "v".into(),
                ty: binding_type!(F32, 2),
            },
        ])
    }
}

#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: Vec4,
    pub local_coord: Vec2,
}

impl Vertex {
    pub fn new(pos: Vec4, local_coord: Vec2) -> Self {
        Self { pos, local_coord }
    }
}

impl StoreBytes for Vertex {
    const SIZE: usize = Vec4::SIZE + Vec2::SIZE;

    fn store_bytes<D: BytesSink>(&self, dst: &mut D) {
        self.pos.store_bytes(dst);
        self.local_coord.store_bytes(dst);
    }
}

impl Attributes for Vertex {
    fn attributes() -> AttributeList {
        Vec4::attributes()
            .with_prefix("pos")
            .chain(Vec2::attributes().with_prefix("local_coord"))
    }
}

#[derive(Clone, Copy)]
pub struct Instance {
    xform: Mat4,
    tex_xform: Affine2,
}

impl Instance {
    pub fn new(xform: Mat4, tex_xform: Affine2) -> Self {
        Self { xform, tex_xform }
    }
}

impl StoreBytes for Instance {
    const SIZE: usize = Vec4::SIZE + Vec2::SIZE;

    fn store_bytes<D: BytesSink>(&self, dst: &mut D) {
        self.xform.store_bytes(dst);
        self.tex_xform.store_bytes(dst);
    }
}

impl Attributes for Instance {
    fn attributes() -> AttributeList {
        Mat4::attributes()
            .with_prefix("xform")
            .chain(Affine2::attributes().with_prefix("tex_xform"))
    }
}
