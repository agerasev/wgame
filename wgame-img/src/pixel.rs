use bytemuck::Pod;
use half::f16;
use rgb::Rgba;

pub trait Pixel: Pod {}

impl Pixel for u8 {}
impl Pixel for Rgba<f16> {}
