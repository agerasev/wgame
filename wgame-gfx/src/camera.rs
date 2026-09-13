use std::{cell::RefCell, num::NonZero};

use glam::{Mat4, Vec2, Vec4};
use rgb::Rgba;
use wgpu::util::DeviceExt;

use crate::{
    Context, Graphics,
    prelude::{Colorable, Transformable},
    types::{Color, Transform, color},
};

#[derive(Clone, Debug)]
pub struct Camera {
    state: Graphics,
    bind_group: RefCell<Option<wgpu::BindGroup>>,
    view: Mat4,
    color: Rgba<f32>,
}

impl Camera {
    pub fn new(state: &Graphics, view: Mat4) -> Self {
        Self {
            state: state.clone(),
            bind_group: RefCell::default(),
            view,
            color: color::WHITE.to_rgba(),
        }
    }

    pub fn view(&self) -> Mat4 {
        self.view
    }
    pub fn color(&self) -> Rgba<f32> {
        self.color
    }

    pub fn world_to_logical(&self, pos: Vec4) -> Vec4 {
        self.view.mul_vec4(pos)
    }
    pub fn logical_to_world(&self, pos: Vec4) -> Vec4 {
        self.view.inverse_or_zero().mul_vec4(pos)
    }

    /// Converts a physical cursor position (top-left origin, Y downward) to
    /// world X/Y using the full target size in physical pixels.
    ///
    /// This is intended for 2D cameras, including transformed
    /// [`crate::Target::physical_camera`] cameras. It unprojects at clip depth
    /// zero; for perspective picking, use [`Self::logical_to_world`] to construct
    /// a ray instead. Positions outside the viewport are allowed.
    /// Returns `None` for an empty viewport, singular camera, or non-finite result.
    ///
    /// ```
    /// # fn pick(camera: &wgame_gfx::Camera) {
    /// let cursor = glam::Vec2::new(120.0, 80.0);
    /// if let Some(world) = camera.screen_to_world(cursor, (800, 600)) {
    ///     // Use world for 2D hit testing.
    ///     assert!(world.is_finite());
    /// }
    /// # }
    /// ```
    pub fn screen_to_world(&self, pos: Vec2, viewport: (u32, u32)) -> Option<Vec2> {
        screen_to_world(self.view, pos, viewport)
    }

    pub(crate) fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("wgame_camera_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZero::new(4 * 4 * 4),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZero::new(4 * 4),
                    },
                    count: None,
                },
            ],
        })
    }
}

impl Transformable for Camera {
    fn transform<X: Transform>(&self, xform: X) -> Self {
        Self {
            view: self.view * xform.to_mat4(),
            bind_group: RefCell::default(),
            ..self.clone()
        }
    }
}

impl Colorable for Camera {
    fn multiply_color<C: Color>(&self, color: C) -> Self {
        let x = self.color;
        let y = color.to_rgba();
        Self {
            color: Rgba {
                r: x.r * y.r,
                g: x.g * y.g,
                b: x.b * y.b,
                a: x.a * y.a,
            },
            bind_group: RefCell::default(),
            ..self.clone()
        }
    }
}

impl Context for Camera {
    fn bind_group(&self) -> wgpu::BindGroup {
        let mut bind_group = self.bind_group.borrow_mut();
        bind_group
            .get_or_insert_with(|| {
                let view_buffer =
                    self.state
                        .device()
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&self.view.to_cols_array()),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });
                let color_buffer =
                    self.state
                        .device()
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&self.color.to_vec4().to_array()),
                            usage: wgpu::BufferUsages::UNIFORM,
                        });
                self.state
                    .device()
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("camera"),
                        layout: self.state.camera_bind_group_layout(),
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &view_buffer,
                                    offset: 0,
                                    size: None,
                                }),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                    buffer: &color_buffer,
                                    offset: 0,
                                    size: None,
                                }),
                            },
                        ],
                    })
            })
            .clone()
    }
}

fn screen_to_world(view: Mat4, pos: Vec2, (width, height): (u32, u32)) -> Option<Vec2> {
    if width == 0 || height == 0 || !view.is_finite() || view.determinant() == 0.0 {
        return None;
    }
    let clip = Vec4::new(
        2.0 * pos.x / width as f32 - 1.0,
        1.0 - 2.0 * pos.y / height as f32,
        0.0,
        1.0,
    );
    let world = view.inverse() * clip;
    let result = world.truncate().truncate() / world.w;
    (world.is_finite() && result.is_finite()).then_some(result)
}

#[cfg(test)]
mod tests {
    use super::screen_to_world;
    use glam::{Mat4, Vec2, Vec3};

    #[test]
    fn physical_pixels_follow_camera_transforms_and_resize() {
        let model = Mat4::from_scale_rotation_translation(
            Vec3::new(3.0, 2.0, 1.0),
            glam::Quat::from_rotation_z(0.4),
            Vec3::new(140.0, 90.0, 0.0),
        );
        for (width, height) in [(800, 600), (1600, 900)] {
            let view =
                Mat4::orthographic_lh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0) * model;
            for point in [Vec2::ZERO, Vec2::new(21.0, -7.0), Vec2::new(-200.0, 800.0)] {
                let pixel = model.transform_point3(point.extend(0.0)).truncate();
                let actual = screen_to_world(view, pixel, (width, height)).unwrap();
                assert!((actual - point).length() < 0.001, "{actual:?} != {point:?}");
            }
        }
    }

    #[test]
    fn rejects_invalid_unprojection() {
        assert_eq!(screen_to_world(Mat4::IDENTITY, Vec2::ZERO, (0, 80)), None);
        assert_eq!(screen_to_world(Mat4::ZERO, Vec2::ZERO, (80, 80)), None);
        assert_eq!(
            screen_to_world(Mat4::IDENTITY, Vec2::splat(f32::NAN), (80, 80)),
            None
        );
        assert_eq!(
            screen_to_world(
                Mat4::from_cols_array(&[f32::INFINITY; 16]),
                Vec2::ZERO,
                (80, 80)
            ),
            None
        );
    }
}
