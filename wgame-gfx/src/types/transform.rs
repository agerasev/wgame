use glam::{Affine2, Affine3A, Mat3, Mat4, Vec3};

pub trait Transform {
    fn to_mat4(self) -> Mat4;
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
