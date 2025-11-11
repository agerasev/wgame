use half::f16;
use rgb::Rgba;
use wgame_image::Pixel;

pub trait Texel: Pixel {
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
