use alloc::{collections::btree_map::BTreeMap, vec::Vec};

use glam::Mat4;

use crate::{Object, State, object::Model};

#[derive(Default)]
struct Instances {
    count: u32,
    data: Vec<u8>,
}

pub struct RenderContext<'a, 'b> {
    pub state: &'b State<'a>,
    pub view: &'b wgpu::TextureView,
}

pub struct Renderer {
    render_passes: BTreeMap<Model, Instances>,
    xform: Mat4,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            render_passes: BTreeMap::new(),
            xform: Mat4::IDENTITY,
        }
    }
}

impl Renderer {
    const BUFFER_ALIGN: u64 = 16;

    pub fn set_xform(&mut self, xform: Mat4) {
        self.xform = xform;
    }

    pub fn enqueue_object<T: Object>(&mut self, object: T) {
        let model = object.model();
        let instances = self.render_passes.entry(model).or_default();
        object.store_instance(self.xform, &mut instances.data);
        instances.count += 1;
    }

    pub fn clear(&mut self) {
        self.render_passes.clear();
    }

    pub fn render(self, ctx: RenderContext<'_, '_>) {
        log::trace!(
            "Render passes with number of instances: {:?}",
            self.render_passes
                .values()
                .map(|inst| inst.count)
                .collect::<Vec<_>>()
        );

        let mut encoder = ctx
            .state
            .0
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let total_buffer_len = self
            .render_passes
            .values()
            .map(|instances| (instances.data.len() as u64).next_multiple_of(Self::BUFFER_ALIGN))
            .sum();

        let instance_buffer = &mut *ctx.state.registry().instance_buffer.borrow_mut();

        if let Some(buffer) = instance_buffer
            && buffer.size() < total_buffer_len
        {
            *instance_buffer = None;
        }

        if instance_buffer.is_none() && total_buffer_len > 0 {
            *instance_buffer = Some(ctx.state.device().create_buffer(&wgpu::BufferDescriptor {
                label: Some("instances"),
                size: total_buffer_len.next_power_of_two(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        let mut offset = 0;
        for instances in self.render_passes.values() {
            if !instances.data.is_empty() {
                let buffer = instance_buffer.as_ref().unwrap();
                ctx.state
                    .queue()
                    .write_buffer(buffer, offset, &instances.data);
                offset += (instances.data.len() as u64).next_multiple_of(Self::BUFFER_ALIGN);
            }
        }

        let attachments = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: ctx.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };

        let mut offset = 0;
        for (model, instances) in self.render_passes {
            let mut renderpass = encoder.begin_render_pass(&attachments);
            {
                renderpass.push_debug_group("prepare");
                renderpass.set_pipeline(&model.pipeline);
                for (i, bind_group) in model.uniforms.iter().enumerate() {
                    renderpass.set_bind_group(i as u32, bind_group, &[]);
                }
                renderpass.set_vertex_buffer(0, model.vertices.vertex_buffer.slice(..));
                if let Some(index_buffer) = &model.vertices.index_buffer {
                    renderpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                }
                if !instances.data.is_empty() {
                    let buffer_len =
                        (instances.data.len() as u64).next_multiple_of(Self::BUFFER_ALIGN);
                    let buffer = instance_buffer.as_ref().unwrap();
                    renderpass.set_vertex_buffer(1, buffer.slice(offset..(offset + buffer_len)));
                    offset += buffer_len;
                }
                renderpass.pop_debug_group();
            }

            renderpass.insert_debug_marker("draw");
            if model.vertices.index_buffer.is_some() {
                renderpass.draw_indexed(0..model.vertices.count, 0, 0..instances.count);
            } else {
                renderpass.draw(0..model.vertices.count, 0..instances.count);
            }
        }

        ctx.state.0.queue.submit(Some(encoder.finish()));
    }
}
