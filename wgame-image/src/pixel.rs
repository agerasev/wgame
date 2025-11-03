use bytemuck::Pod;
use half::f16;
use rgb::Rgba;

pub trait Pixel: Pod + Default {}

impl Pixel for u8 {}
impl Pixel for Rgba<u8> {}
impl Pixel for f16 {}
impl Pixel for Rgba<f16> {}
