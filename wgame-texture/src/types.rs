use half::f16;
use rgb::Rgba;

pub type Rect = guillotiere::euclid::default::Box2D<u32>;

pub trait Texel: Copy + bytemuck::Pod {
    fn is_format_supported(format: wgpu::TextureFormat) -> bool;
}

impl Texel for u8 {
    fn is_format_supported(format: wgpu::TextureFormat) -> bool {
        use wgpu::TextureFormat::*;
        matches!(format, R8Uint | R8Unorm)
    }
}

impl Texel for Rgba<u8> {
    fn is_format_supported(format: wgpu::TextureFormat) -> bool {
        use wgpu::TextureFormat::*;
        matches!(format, Rgba8Uint | Rgba8Unorm | Rgba8UnormSrgb)
    }
}

impl Texel for f16 {
    fn is_format_supported(format: wgpu::TextureFormat) -> bool {
        use wgpu::TextureFormat::*;
        matches!(format, R16Float)
    }
}

impl Texel for Rgba<f16> {
    fn is_format_supported(format: wgpu::TextureFormat) -> bool {
        use wgpu::TextureFormat::*;
        matches!(format, Rgba16Float)
    }
}
