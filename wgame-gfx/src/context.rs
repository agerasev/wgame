use glam::Mat4;
use half::f16;
use rgb::Rgba;

use crate::types::Transform;

#[derive(Clone, Debug)]
pub struct Context {
    pub view: Mat4,
    pub color: Rgba<f16>,
}

impl Context {
    pub fn transform(&self, xform: impl Transform) -> Self {
        Self {
            view: self.view * xform.to_mat4(),
            ..self.clone()
        }
    }
}
