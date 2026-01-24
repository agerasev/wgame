use std::{cell::RefCell, num::NonZero};

use glam::{Mat4, Vec4};
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
    fn mul_color<C: Color>(&self, color: C) -> Self {
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
