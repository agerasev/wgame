use glam::{Vec2, Vec3, Vec3A, Vec4};

pub trait Position {
    fn to_xyzw(self) -> Vec4;
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
