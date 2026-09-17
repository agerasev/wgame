//! Attachment clears for whole targets; scissored draws for borrowed regions.
use std::{
    hash::{Hash, Hasher},
    sync::{Arc, OnceLock},
};

use rgb::Rgba;
use wgpu::util::DeviceExt;

use crate::{DEPTH_FORMAT, Target, viewport::Region};

// Cache identity must stay stable when the pipelines are initialized: Graphics
// participates in resource equality/hash keys. Clones share this cache.
#[derive(Clone, Debug, Default)]
pub(crate) struct ClearCache(Arc<OnceLock<ClearPipelines>>);
impl PartialEq for ClearCache {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for ClearCache {}
impl Hash for ClearCache {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
impl ClearCache {
    fn get(&self, device: &wgpu::Device, format: wgpu::TextureFormat) -> &ClearPipelines {
        self.0.get_or_init(|| ClearPipelines::new(device, format))
    }
}

#[derive(Debug)]
struct ClearPipelines {
    color_layout: wgpu::BindGroupLayout,
    color: wgpu::RenderPipeline,
    depth: wgpu::RenderPipeline,
}
impl ClearPipelines {
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("viewport clear"),
            source: wgpu::ShaderSource::Wgsl(include_str!("clear.wgsl").into()),
        });
        let color_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("viewport clear color"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZero::new(16),
                },
                count: None,
            }],
        });
        let fragment_entry = match format.sample_type(None, None) {
            Some(wgpu::TextureSampleType::Uint) => "color_uint",
            Some(wgpu::TextureSampleType::Sint) => "color_sint",
            _ => "color_float",
        };
        let pipeline = |color: bool| {
            let color_layouts = [Some(&color_layout)];
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("viewport clear"),
                bind_group_layouts: if color { &color_layouts } else { &[] },
                immediate_size: 0,
            });
            let targets = [Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })];
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("viewport clear"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertex_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: color.then_some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(fragment_entry),
                    compilation_options: Default::default(),
                    targets: &targets,
                }),
                primitive: Default::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::Always),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        Self {
            color: pipeline(true),
            depth: pipeline(false),
            color_layout,
        }
    }
}

pub(crate) fn clear(target: &mut (impl Target + ?Sized), color: Option<Rgba<f32>>) {
    let color = color.map(|c| {
        if target.premultiplied_alpha() {
            Rgba::new(c.r * c.a, c.g * c.a, c.b * c.a, c.a)
        } else {
            c
        }
    });
    let region = Region {
        origin: target.origin(),
        size: target.size(),
    };
    let view = target.view().clone();
    let depth = target.depth_view().clone();
    let extent = view.texture().size();
    let whole = region.origin == (0, 0) && region.size == (extent.width, extent.height);
    let state = target.state().clone();
    let pipelines = (!whole).then(|| state.clear.get(state.device(), state.format()));
    // A fresh immutable snapshot preserves different clear colors encoded before
    // one submission. Rewriting one shared uniform would recolor earlier clears.
    let bind_group = pipelines.zip(color).map(|(pipelines, color)| {
        let uniform = state
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("viewport clear color"),
                contents: bytemuck::cast_slice(&[color.r, color.g, color.b, color.a]),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        state
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("viewport clear color"),
                layout: &pipelines.color_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                }],
            })
    });
    let attachments = [color.map(|color| wgpu::RenderPassColorAttachment {
        view: &view,
        resolve_target: None,
        depth_slice: None,
        ops: wgpu::Operations {
            load: if whole {
                wgpu::LoadOp::Clear(wgpu::Color {
                    r: color.r as f64,
                    g: color.g as f64,
                    b: color.b as f64,
                    a: color.a as f64,
                })
            } else {
                wgpu::LoadOp::Load
            },
            store: wgpu::StoreOp::Store,
        },
    })];
    let mut pass = target
        .encoder()
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("target clear"),
            color_attachments: if color.is_some() { &attachments } else { &[] },
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth,
                depth_ops: Some(wgpu::Operations {
                    load: if whole {
                        wgpu::LoadOp::Clear(1.0)
                    } else {
                        wgpu::LoadOp::Load
                    },
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
    if let Some(pipelines) = pipelines {
        region.apply(&mut pass);
        pass.set_pipeline(if color.is_some() {
            &pipelines.color
        } else {
            &pipelines.depth
        });
        if let Some(bind_group) = &bind_group {
            pass.set_bind_group(0, bind_group, &[]);
        }
        pass.draw(0..3, 0..1);
    }
}
