use glam::{Vec2, Vec3, Vec3A, Vec4};
use rgb::{Rgb, Rgba};

pub trait Position {
    fn to_xyzw(self) -> Vec4;
}

pub trait Color {
    fn to_rgba(self) -> Rgba<f32>;
}

impl Position for Vec2 {
    fn to_xyzw(self) -> Vec4 {
        Vec4::from((self, 0.0, 1.0))
    }
}

impl Position for Vec3 {
    fn to_xyzw(self) -> Vec4 {
        Vec4::from((self, 1.0))
    }
}

impl Position for Vec3A {
    fn to_xyzw(self) -> Vec4 {
        Vec4::from((self, 1.0))
    }
}

impl Position for Vec4 {
    fn to_xyzw(self) -> Vec4 {
        self
    }
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
