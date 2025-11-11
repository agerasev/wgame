use std::marker::PhantomData;

use half::f16;

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

pub trait StoreBytes {
    fn store_bytes(&self, dst: &mut BytesSink);

    fn to_bytes(&self) -> Vec<u8> {
        let mut dst = BytesSink::default();
        self.store_bytes(&mut dst);
        dst.data
    }
}

impl<T> StoreBytes for PhantomData<T> {
    fn store_bytes(&self, _: &mut BytesSink) {}
}

impl StoreBytes for () {
    fn store_bytes(&self, _: &mut BytesSink) {}
}

impl<T: StoreBytes, const N: usize> StoreBytes for [T; N] {
    fn store_bytes(&self, dst: &mut BytesSink) {
        for item in self {
            item.store_bytes(dst);
        }
    }
}

macro_rules! impl_store_pod {
    ($type:ty) => {
        impl StoreBytes for $type {
            fn store_bytes(&self, dst: &mut BytesSink) {
                dst.push_bytes(bytemuck::bytes_of(self));
            }
        }
    };
}

impl_store_pod!(glam::Mat4);
impl_store_pod!(glam::Mat3);
impl_store_pod!(glam::Mat2);
impl_store_pod!(glam::Vec4);
impl_store_pod!(glam::Vec3);
impl_store_pod!(glam::Vec2);
impl_store_pod!(rgb::Rgba<f32>);
impl_store_pod!(rgb::Rgba<f16>);
impl_store_pod!(f32);
impl_store_pod!(f16);

impl StoreBytes for glam::Affine2 {
    fn store_bytes(&self, dst: &mut BytesSink) {
        dst.push_bytes(bytemuck::bytes_of(&self.matrix2));
        dst.push_bytes(bytemuck::bytes_of(&self.translation));
    }
}
