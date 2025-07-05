use glam::{Affine2, Affine3A, Mat3, Mat4, Vec2, Vec3, Vec3A, Vec4};
use rgb::{Rgb, Rgba};

pub trait Position {
    fn to_xyzw(self) -> Vec4;
}

pub trait Transform {
    fn to_mat4(self) -> Mat4;
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

impl Transform for Mat4 {
    fn to_mat4(self) -> Mat4 {
        self
    }
}

impl Transform for Affine3A {
    fn to_mat4(self) -> Mat4 {
        self.into()
    }
}

impl Transform for Affine2 {
    fn to_mat4(self) -> Mat4 {
        let m = self.matrix2;
        let v = self.translation;
        Affine3A::from_mat3_translation(
            Mat3::from_cols(
                Vec3::from((m.x_axis, 0.0)),
                Vec3::from((m.y_axis, 0.0)),
                Vec3::Z,
            ),
            Vec3::from((v, 0.0)),
        )
        .to_mat4()
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
