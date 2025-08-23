use glam::Mat4;
use half::f16;
use rgb::Rgba;

use crate::types::{Color, Transform};

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
    pub fn color(&self, color: impl Color) -> Self {
        let x = self.color;
        let y = color.to_rgba();
        Self {
            color: Rgba {
                r: x.r * y.r,
                g: x.g * y.g,
                b: x.b * y.b,
                a: x.a * y.a,
            },
            ..self.clone()
        }
    }
}
