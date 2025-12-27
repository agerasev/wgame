use glam::{Affine2, Affine3A, Mat3, Mat4, Vec3};

pub trait Transform: Copy {
    fn to_affine3(self) -> Affine3A;

    fn to_mat4(self) -> Mat4 {
        self.to_affine3().into()
    }
}

impl Transform for Mat3 {
    fn to_affine3(self) -> Affine3A {
        Affine3A::from_mat3(self)
    }
}

impl Transform for Affine3A {
    fn to_affine3(self) -> Affine3A {
        self
    }
}

impl Transform for Affine2 {
    fn to_affine3(self) -> Affine3A {
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
    }
}
