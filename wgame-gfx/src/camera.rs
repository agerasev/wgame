use glam::Mat4;
use half::f16;
use rgb::Rgba;

use crate::types::{Color, Transform, color};

#[derive(Clone, Debug)]
pub struct Camera {
    pub view: Mat4,
    pub color: Rgba<f16>,
    pub y_flip: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            view: Mat4::IDENTITY,
            color: color::WHITE.to_rgba(),
            y_flip: false,
        }
    }
}

impl Camera {
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
