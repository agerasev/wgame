use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::mem::replace;

use anyhow::Result;

use crate::{binding::BindingType, binding_type};

#[derive(Clone, Debug)]
pub struct AttributeInfo {
    name: String,
    ty: BindingType,
}

#[derive(Clone, Default, Debug)]
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

macro_rules! impl_attributes_pod {
    ($type:ty, $layout:expr) => {
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

impl_attributes_pod!(glam::Mat4, (0..4).map(|_| binding_type!(F32, 4)));
impl_attributes_pod!(glam::Mat3, (0..3).map(|_| binding_type!(F32, 4)));
impl_attributes_pod!(glam::Mat2, [binding_type!(F32, 4)]);
impl_attributes_pod!(glam::Vec4, [binding_type!(F32, 4)]);
impl_attributes_pod!(glam::Vec3, [binding_type!(F32, 3)]);
impl_attributes_pod!(glam::Vec2, [binding_type!(F32, 2)]);
impl_attributes_pod!(f32, [binding_type!(F32)]);

impl Attributes for glam::Affine2 {
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
