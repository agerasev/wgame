use alloc::{collections::btree_map::BTreeMap, vec::Vec};

use glam::Mat4;
use half::f16;
use rgb::{ComponentMap, Rgba};

use crate::{Object, State, object::Model, types::Color};

#[derive(Default)]
struct Instances {
    count: u32,
    data: Vec<u8>,
}

pub struct RenderContext<'b> {
    view: &'b wgpu::TextureView,
    instance_buffer: &'b mut Option<wgpu::Buffer>,
}

pub struct Renderer<'a> {
    state: State<'a>,
    clear_color: Option<Rgba<f16>>,
    render_passes: BTreeMap<Model, Instances>,
}

impl Renderer<'_> {
    pub fn set_clear_color(&mut self, color: impl Color) {
        self.clear_color = Some(color.to_rgba());
    }

    pub fn enqueue_object<T: Object>(&mut self, object: T, xform: Mat4) {
        let model = object.model();
        let instances = self.render_passes.entry(model).or_default();
        object.store_instance(xform, &mut instances.data);
        instances.count += 1;
    }

    pub fn render(self, context: RenderContext) {
        let mut encoder = self
            .state
            .0
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        if let Some(color) = self.clear_color {
            let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: context.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear({
                            let Rgba { r, g, b, a } = color.map(f16::to_f64);
                            wgpu::Color { r, g, b, a }
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        if let Some(max_buffer_len) = self
            .render_passes
            .values()
            .map(|instances| instances.data.len())
            .max()
        {}

        let instance_buffer = context.instance_buffer.get_or_insert_with(|| {});

        let attachments = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: context.view,
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
                renderpass.set_vertex_buffer(1, self.instances.buffer.slice(..));
                renderpass.pop_debug_group();
            }
            renderpass.insert_debug_marker("draw");
            if model.vertices.index_buffer.is_some() {
                renderpass.draw_indexed(0..model.vertices.count, 0, 0..self.instances.count);
            } else {
                renderpass.draw(0..model.vertices.count, 0..self.instances.count);
            }
        }

        self.state.0.queue.submit(Some(encoder.finish()));
    }
}
