use glam::{Vec3, Vec4};
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
    fn to_rgba(self) -> Rgba<f32>;
}

impl Color for Rgb<f32> {
    fn to_rgba(self) -> Rgba<f32> {
        Rgba::new(self.r, self.g, self.b, 1.0)
    }
}

impl Color for Rgba<f32> {
    fn to_rgba(self) -> Rgba<f32> {
        self
    }
}

impl Color for Vec3 {
    fn to_rgba(self) -> Rgba<f32> {
        Rgba {
            r: self.x,
            g: self.y,
            b: self.z,
            a: 1.0,
        }
    }
}

impl Color for Vec4 {
    fn to_rgba(self) -> Rgba<f32> {
        Rgba {
            r: self.x,
            g: self.y,
            b: self.z,
            a: self.w,
        }
    }
}
