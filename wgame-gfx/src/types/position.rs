use glam::{Vec2, Vec3, Vec3A};

pub trait Position: Copy {
    fn to_xyz(self) -> Vec3;
}

impl Position for Vec2 {
    fn to_xyz(self) -> Vec3 {
        (self, 0.0).into()
    }
}

impl Position for Vec3 {
    fn to_xyz(self) -> Vec3 {
        self
    }
}

impl Position for Vec3A {
    fn to_xyz(self) -> Vec3 {
        self.into()
    }
}
