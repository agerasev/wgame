use glam::{Vec3, Vec4};
use half::f16;
use rgb::{Rgb, Rgba};

pub const BLACK: Rgb<f32> = Rgb::new(0.0, 0.0, 0.0);
pub const RED: Rgb<f32> = Rgb::new(1.0, 0.0, 0.0);
pub const YELLOW: Rgb<f32> = Rgb::new(1.0, 1.0, 0.0);
pub const GREEN: Rgb<f32> = Rgb::new(0.0, 1.0, 0.0);
pub const CYAN: Rgb<f32> = Rgb::new(0.0, 1.0, 1.0);
pub const BLUE: Rgb<f32> = Rgb::new(0.0, 0.0, 1.0);
pub const MAGENTA: Rgb<f32> = Rgb::new(1.0, 0.0, 1.0);
pub const WHITE: Rgb<f32> = Rgb::new(1.0, 1.0, 1.0);

pub trait Color {
    fn to_rgba(self) -> Rgba<f16>;
}

impl Color for Rgb<f32> {
    fn to_rgba(self) -> Rgba<f16> {
        Rgba::new(
            f16::from_f32(self.r),
            f16::from_f32(self.g),
            f16::from_f32(self.b),
            f16::ONE,
        )
    }
}

impl Color for Rgba<f32> {
    fn to_rgba(self) -> Rgba<f16> {
        Rgba::new(
            f16::from_f32(self.r),
            f16::from_f32(self.g),
            f16::from_f32(self.b),
            f16::from_f32(self.a),
        )
    }
}

impl Color for Rgb<f16> {
    fn to_rgba(self) -> Rgba<f16> {
        Rgba::new(self.r, self.g, self.b, f16::ONE)
    }
}

impl Color for Rgba<f16> {
    fn to_rgba(self) -> Rgba<f16> {
        self
    }
}

impl Color for Vec3 {
    fn to_rgba(self) -> Rgba<f16> {
        Rgba {
            r: f16::from_f32(self.x),
            g: f16::from_f32(self.y),
            b: f16::from_f32(self.z),
            a: f16::ONE,
        }
    }
}

impl Color for Vec4 {
    fn to_rgba(self) -> Rgba<f16> {
        Rgba {
            r: f16::from_f32(self.x),
            g: f16::from_f32(self.y),
            b: f16::from_f32(self.z),
            a: f16::from_f32(self.w),
        }
    }
}
