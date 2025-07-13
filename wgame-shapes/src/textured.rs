use alloc::{vec, vec::Vec};

use glam::Mat4;
use wgpu::util::DeviceExt;

use wgame_gfx::{Object, Texture, types::Transform};

use crate::Shape;

pub struct Textured<'a, T: Shape<'a>> {
    shape: T,
    texture: Texture<'a>,

    xform_buffer: wgpu::Buffer,
    tex_xform_buffer: wgpu::Buffer,

    vertex_bind_group: wgpu::BindGroup,
    fragment_bind_group: wgpu::BindGroup,
}

impl<'a, T: Shape<'a>> Textured<'a, T> {
    pub fn new(shape: T, texture: Texture<'a>) -> Self {
        let device = shape.state().device();

        let xform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("transform"),
            contents: bytemuck::cast_slice(Mat4::IDENTITY.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let tex_xform = texture.coord_xform().to_mat4();
        let tex_xform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tex_transform"),
            contents: bytemuck::cast_slice(tex_xform.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let pipeline = shape.pipeline();
        let uniforms = shape.uniforms();
        let vertex_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: xform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: tex_xform_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });
        let fragment_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.get_bind_group_layout(1),
            entries: &([
                wgpu::BindingResource::TextureView(texture.view()),
                wgpu::BindingResource::Sampler(texture.sampler()),
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
        });

        Self {
            shape,
            texture,
            xform_buffer,
            tex_xform_buffer,
            vertex_bind_group,
            fragment_bind_group,
        }
    }

    fn get_uniforms(&self, xform: Mat4) -> Vec<wgpu::BindGroup> {
        let final_xform = xform * self.shape.xform();
        self.shape.state().queue().write_buffer(
            &self.xform_buffer,
            0,
            bytemuck::cast_slice(final_xform.as_ref()),
        );

        vec![
            self.vertex_bind_group.clone(),
            self.fragment_bind_group.clone(),
        ]
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
        let uniforms = self.get_uniforms(xform);
        let mut renderpass = encoder.begin_render_pass(attachments);
        {
            renderpass.push_debug_group("prepare");
            renderpass.set_pipeline(&self.shape.pipeline());
            for (i, bind_group) in uniforms.into_iter().enumerate() {
                renderpass.set_bind_group(i as u32, &bind_group, &[]);
            }
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
