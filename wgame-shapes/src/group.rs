use alloc::vec::Vec;

use glam::Mat4;

use wgame_gfx::Object;

use crate::{Instances, Vertices, shader::UniformInfo};

pub trait UniformType {
    fn info(&self) -> Vec<UniformInfo>;
}

pub trait InstanceType {
    fn info(&self) -> Vec<UniformInfo>;
}

pub trait Type {
    type Uniform: UniformType;
    type Instance: Instance;
}
pub trait Instance {
    fn store(&self, xform: Mat4, dst: &mut Vec<u8>);
}

/// Single render pass
pub struct Group {
    vertices: Vertices,
    instances: Instances,
    uniforms: Vec<wgpu::BindGroup>,
    pipeline: wgpu::RenderPipeline,
}

impl Object for Group {
    fn render(
        &self,
        attachments: &wgpu::RenderPassDescriptor<'_>,
        encoder: &mut wgpu::CommandEncoder,
        xform: Mat4,
    ) {
        let mut renderpass = encoder.begin_render_pass(attachments);
        {
            renderpass.push_debug_group("prepare");
            renderpass.set_pipeline(&self.pipeline);
            for (i, bind_group) in self.uniforms.iter().enumerate() {
                renderpass.set_bind_group(i as u32, bind_group, &[]);
            }
            renderpass.set_vertex_buffer(0, self.vertices.vertex_buffer.slice(..));
            if let Some(index_buffer) = &self.vertices.index_buffer {
                renderpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            }
            renderpass.set_vertex_buffer(1, self.instances.buffer.slice(..));
            renderpass.pop_debug_group();
        }
        renderpass.insert_debug_marker("draw");
        if self.vertices.index_buffer.is_some() {
            renderpass.draw_indexed(0..self.vertices.count, 0, 0..self.instances.count);
        } else {
            renderpass.draw(0..self.vertices.count, 0..self.instances.count);
        }
    }
}
