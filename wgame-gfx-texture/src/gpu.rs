//! GPU allocation, sampler bindings and uploads shared by atlas and render textures.
use crate::{FilterMode, Texel, TextureSettings, TexturingState, sampling::TextureBinding};
use euclid::default::{Point2D, Rect, Size2D};
use hashbrown::HashMap;
use std::cell::RefCell;
use wgame_image::{ImageBase, ImageRead, ImageSlice};
use wgpu::util::DeviceExt;

#[derive(Clone)]
pub(crate) struct GpuTexture {
    state: TexturingState,
    extent: wgpu::Extent3d,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    bind_groups: RefCell<HashMap<TextureSettings, wgpu::BindGroup>>,
    sampling_info: wgpu::Buffer,
}

impl GpuTexture {
    pub(crate) fn new(
        state: &TexturingState,
        size: Size2D<u32>,
        format: wgpu::TextureFormat,
    ) -> Self {
        let state = state.clone();
        let device = state.device();

        let extent = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        Self::from_texture(&state, texture, false)
    }

    pub(crate) fn from_texture(
        state: &TexturingState,
        texture: wgpu::Texture,
        premultiplied: bool,
    ) -> Self {
        Self {
            state: state.clone(),
            extent: texture.size(),
            view: texture.create_view(&Default::default()),
            texture,
            bind_groups: RefCell::default(),
            sampling_info: state
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("texture alpha representation"),
                    contents: bytemuck::cast_slice(&[u32::from(premultiplied), 0, 0, 0]),
                    usage: wgpu::BufferUsages::UNIFORM,
                }),
        }
    }

    fn get_bind_group(&self, settings: TextureSettings) -> wgpu::BindGroup {
        self.bind_groups
            .borrow_mut()
            .entry(settings)
            .or_insert_with(|| {
                let format = self.texture.format();
                match format.sample_type(None, None) {
                    Some(wgpu::TextureSampleType::Uint) => {
                        assert_eq!(settings.mag_filter, FilterMode::Nearest);
                        self.state
                            .device()
                            .create_bind_group(&wgpu::BindGroupDescriptor {
                                layout: &self.state.uint_bind_group_layout,
                                entries: &[wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&self.view),
                                }],
                                label: None,
                            })
                    }
                    Some(wgpu::TextureSampleType::Float { filterable: true }) => self
                        .state
                        .device()
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &self.state.float_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(&self.view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(
                                        match settings.mag_filter {
                                            FilterMode::Nearest => &self.state.nearest_sampler,
                                            FilterMode::Linear => &self.state.linear_sampler,
                                        },
                                    ),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: self.sampling_info.as_entire_binding(),
                                },
                            ],
                            label: None,
                        }),
                    _ => panic!("Unsupported texture format: {format:?}"),
                }
            })
            .clone()
    }

    pub(crate) fn write<T: Texel>(&self, data: ImageSlice<T>, dst: Point2D<u32>) {
        let format = self.texture.format();
        assert!(T::is_format_supported(format));

        let size = data.size();
        let dst_rect = Rect { origin: dst, size };
        assert!(dst_rect.max_x() <= self.extent.width && dst_rect.max_y() <= self.extent.height);

        let bytes_per_block = format.block_copy_size(None).unwrap() as usize;
        assert_eq!(size_of::<T>(), bytes_per_block);

        self.state.queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: dst.x,
                    y: dst.y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(data.data()),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(data.stride() * size_of::<T>() as u32),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
        );
    }
}

impl TextureBinding for GpuTexture {
    fn bind_group(&self, settings: TextureSettings) -> wgpu::BindGroup {
        self.get_bind_group(settings)
    }
    fn bind_group_layout(&self) -> wgpu::BindGroupLayout {
        self.state.bind_group_layout(self.texture.format())
    }
}
