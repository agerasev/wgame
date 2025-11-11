use std::marker::PhantomData;

use half::f16;

use crate::binding::{Binding, BindingList, binding_type};

#[derive(Default, Clone)]
pub struct BytesSink {
    data: Vec<u8>,
}

impl BytesSink {
    pub fn push_bytes(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }
    pub fn into_data(self) -> Vec<u8> {
        self.data
    }
}

pub trait Attribute: 'static {
    fn bindings() -> BindingList;

    const SIZE: usize;

    fn store(&self, dst: &mut BytesSink);

    fn to_bytes(&self) -> Vec<u8> {
        let mut dst = BytesSink::default();
        self.store(&mut dst);
        dst.data
    }
}

impl<T: 'static> Attribute for PhantomData<T> {
    fn bindings() -> BindingList {
        BindingList::default()
    }

    const SIZE: usize = 0;

    fn store(&self, _: &mut BytesSink) {}
}

impl Attribute for () {
    fn bindings() -> BindingList {
        BindingList::default()
    }

    const SIZE: usize = 0;

    fn store(&self, _: &mut BytesSink) {}
}

impl<T: Attribute, const N: usize> Attribute for [T; N] {
    fn bindings() -> BindingList {
        let mut bindings = BindingList::default();
        for i in 0..N {
            bindings = bindings.chain(T::bindings().with_prefix(&format!("{i}")))
        }
        bindings
    }

    const SIZE: usize = T::SIZE * N;

    fn store(&self, dst: &mut BytesSink) {
        for item in self {
            item.store(dst);
        }
    }
}

macro_rules! impl_attributes_pod {
    ($type:ty, $layout:expr) => {
        impl Attribute for $type
        where
            $type: bytemuck::Pod,
        {
            fn bindings() -> BindingList {
                let mut bindings = BindingList::default();
                let count = $layout.len();
                for (i, ty) in $layout.into_iter().enumerate() {
                    bindings.push(Binding {
                        name: if count > 1 {
                            format!("{i}")
                        } else {
                            String::new()
                        },
                        ty,
                    })
                }
                bindings
            }

            const SIZE: usize = size_of::<$type>();

            fn store(&self, dst: &mut BytesSink) {
                dst.push_bytes(bytemuck::bytes_of(self));
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
impl_attributes_pod!(rgb::Rgba<f32>, [binding_type!(F32, 4)]);
impl_attributes_pod!(rgb::Rgba<f16>, [binding_type!(F16, 4)]);
impl_attributes_pod!(f32, [binding_type!(F32)]);
impl_attributes_pod!(f16, [binding_type!(F16)]);

impl Attribute for glam::Affine2 {
    fn bindings() -> BindingList {
        BindingList::from_iter([
            Binding {
                name: "m".into(),
                ty: binding_type!(F32, 4),
            },
            Binding {
                name: "v".into(),
                ty: binding_type!(F32, 2),
            },
        ])
    }

    const SIZE: usize = size_of::<glam::Mat2>() + size_of::<glam::Vec2>();

    fn store(&self, dst: &mut BytesSink) {
        dst.push_bytes(bytemuck::bytes_of(&self.matrix2));
        dst.push_bytes(bytemuck::bytes_of(&self.translation));
    }
}
