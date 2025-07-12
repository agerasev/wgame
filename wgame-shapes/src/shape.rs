use alloc::vec::Vec;

use glam::{Affine2, Mat4, Vec2};
use wgpu::util::DeviceExt;

use wgame_gfx::{
    Object, State, Texture, Transformed,
    types::{Color, Transform},
};

pub struct Vertices {
    pub count: u32,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
}

pub struct Uniforms {
    pub vertex: wgpu::BindGroup,
    pub fragment: wgpu::BindGroup,
}

pub trait Shape<'a> {
    fn state(&self) -> &State<'a>;
    fn vertices(&self) -> Vertices;
    fn uniforms(&self) -> Vec<wgpu::Buffer> {
        Vec::new()
    }
    fn transformation(&self) -> Mat4 {
        Mat4::IDENTITY
    }
    fn pipeline(&self) -> wgpu::RenderPipeline;
}

pub trait ShapeExt<'a>: Shape<'a> + Sized {
    fn transform<T: Transform>(self, xform: T) -> Transformed<Self> {
        Transformed {
            inner: self,
            xform: xform.to_mat4(),
        }
    }

    fn color<T: Color>(self, color: T) -> Textured<'a, Self> {
        let pixel = Texture::with_data(self.state(), (1, 1), &[color.to_rgba()]);
        self.texture(pixel)
    }

    fn gradient<T: Color>(self, colors: [[T; 2]; 2]) -> Textured<'a, Self> {
        let colors = colors.map(|row| row.map(|color| color.to_rgba()));
        let pixels_2x2 = Texture::with_data(self.state(), (2, 2), colors.as_flattened())
            .transform_coord(Affine2::from_scale_angle_translation(
                Vec2::new(0.5, 0.5),
                0.0,
                Vec2::new(0.25, 0.25),
            ));
        self.texture(pixels_2x2)
    }

    fn texture(self, texture: impl Into<Texture<'a>>) -> Textured<'a, Self> {
        Textured {
            shape: self,
            texture: texture.into(),
        }
    }
}

impl<'a, T: Shape<'a>> ShapeExt<'a> for T {}

impl<'a, T: Shape<'a>> Shape<'a> for Transformed<T> {
    fn state(&self) -> &State<'a> {
        self.inner.state()
    }

    fn vertices(&self) -> Vertices {
        self.inner.vertices()
    }

    fn uniforms(&self) -> Vec<wgpu::Buffer> {
        self.inner.uniforms()
    }

    fn transformation(&self) -> Mat4 {
        self.xform * self.inner.transformation()
    }

    fn pipeline(&self) -> wgpu::RenderPipeline {
        self.inner.pipeline()
    }
}

pub struct Textured<'a, T: Shape<'a>> {
    pub shape: T,
    pub texture: Texture<'a>,
}

impl<'a, T: Shape<'a>> Textured<'a, T> {
    fn create_uniforms(&self, xform: Mat4) -> Uniforms {
        let device = &self.shape.state().device();
        let final_xform = xform * self.shape.transformation();
        let xform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("transform"),
            contents: bytemuck::cast_slice(final_xform.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let tex_xform = self.texture.coord_xform().to_mat4();
        let text_xform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tex_transform"),
            contents: bytemuck::cast_slice(tex_xform.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let uniforms = self.shape.uniforms();
        let pipeline = self.shape.pipeline();
        Uniforms {
            vertex: device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &pipeline.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: xform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: text_xform_buffer.as_entire_binding(),
                    },
                ],
                label: None,
            }),
            fragment: device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &pipeline.get_bind_group_layout(1),
                entries: &([
                    wgpu::BindingResource::TextureView(self.texture.view()),
                    wgpu::BindingResource::Sampler(self.texture.sampler()),
                ]
                .into_iter())
                .chain(uniforms.iter().map(|buffer| buffer.as_entire_binding()))
                .enumerate()
                .map(|(i, resource)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource,
                })
                .collect::<Vec<_>>(),
                label: None,
            }),
        }
    }
}

impl<'a, T: Shape<'a>> Object for Textured<'a, T> {
    fn render(
        &self,
        attachments: &wgpu::RenderPassDescriptor<'_>,
        encoder: &mut wgpu::CommandEncoder,
        xform: Mat4,
    ) {
        let vertices = self.shape.vertices();
        let uniforms = self.create_uniforms(xform);
        let mut renderpass = encoder.begin_render_pass(attachments);
        {
            renderpass.push_debug_group("prepare");
            renderpass.set_pipeline(&self.shape.pipeline());
            renderpass.set_bind_group(0, &uniforms.vertex, &[]);
            renderpass.set_bind_group(1, &uniforms.fragment, &[]);
            renderpass.set_vertex_buffer(0, vertices.vertex_buffer.slice(..));
            if let Some(index_buffer) = &vertices.index_buffer {
                renderpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            }
            renderpass.pop_debug_group();
        }
        renderpass.insert_debug_marker("draw");
        if vertices.index_buffer.is_some() {
            renderpass.draw_indexed(0..vertices.count, 0, 0..1);
        } else {
            renderpass.draw(0..vertices.count, 0..1);
        }
    }
}
