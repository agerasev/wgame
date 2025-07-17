use core::marker::PhantomData;

use alloc::vec::Vec;

pub trait BytesSink {
    fn push_bytes(&mut self, data: &[u8]);
}

impl BytesSink for Vec<u8> {
    fn push_bytes(&mut self, data: &[u8]) {
        self.extend_from_slice(data);
    }
}

pub trait StoreBytes {
    const SIZE: usize;

    fn store_bytes<D: BytesSink>(&self, dst: &mut D);

    fn to_bytes(&self) -> Vec<u8> {
        let mut dst = Vec::new();
        self.store_bytes(&mut dst);
        dst
    }
}

impl<T> StoreBytes for PhantomData<T> {
    const SIZE: usize = 0;
    fn store_bytes<D: BytesSink>(&self, _: &mut D) {}
}

impl StoreBytes for () {
    const SIZE: usize = 0;
    fn store_bytes<D: BytesSink>(&self, _: &mut D) {}
}

impl<T: StoreBytes, const N: usize> StoreBytes for [T; N] {
    const SIZE: usize = T::SIZE * N;

    fn store_bytes<D: BytesSink>(&self, dst: &mut D) {
        for item in self {
            item.store_bytes(dst);
        }
    }
}

macro_rules! impl_store_pod {
    ($type:ty) => {
        impl StoreBytes for $type {
            const SIZE: usize = size_of::<$type>();

            fn store_bytes<D: BytesSink>(&self, dst: &mut D) {
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
impl_store_pod!(f32);

impl StoreBytes for glam::Affine2 {
    const SIZE: usize = size_of::<glam::Mat2>() + size_of::<glam::Vec2>();

    fn store_bytes<D: BytesSink>(&self, dst: &mut D) {
        dst.push_bytes(bytemuck::bytes_of(&self.matrix2));
        dst.push_bytes(bytemuck::bytes_of(&self.translation));
    }
}
