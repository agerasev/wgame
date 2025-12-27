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

pub trait Color: Copy {
    fn to_rgba(self) -> Rgba<f32>;

    fn to_rgba_f16(self) -> Rgba<f16> {
        let this = self.to_rgba();
        Rgba::new(
            f16::from_f32(this.r),
            f16::from_f32(this.g),
            f16::from_f32(this.b),
            f16::from_f32(this.a),
        )
    }

    fn to_vec4(self) -> Vec4 {
        let c = self.to_rgba();
        Vec4::new(c.r, c.g, c.b, c.a)
    }

    fn mul(self, other: impl Color) -> Rgba<f32> {
        (self.to_vec4() * other.to_vec4()).to_rgba()
    }

    fn mix(self, other: impl Color, other_weight: f32) -> Rgba<f32> {
        (self.to_vec4() * (1.0 - other_weight) + other.to_vec4() * other_weight).to_rgba()
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

impl Color for Rgb<f16> {
    fn to_rgba(self) -> Rgba<f32> {
        Rgba::new(self.r.to_f32(), self.g.to_f32(), self.b.to_f32(), 1.0)
    }
}

impl Color for Rgba<f16> {
    fn to_rgba(self) -> Rgba<f32> {
        Rgba::new(
            self.r.to_f32(),
            self.g.to_f32(),
            self.b.to_f32(),
            self.a.to_f32(),
        )
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

    fn to_vec4(self) -> Vec4 {
        Vec4::from((self, 1.0))
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

    fn to_vec4(self) -> Vec4 {
        self
    }
}
