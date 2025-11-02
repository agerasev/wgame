use alloc::{collections::vec_deque::VecDeque, rc::Rc};
use core::cell::RefCell;
use euclid::default::{Point2D, Rect, Size2D};
use glam::Affine2;
use wgame_img::{ImageBase, ImageRead, ImageSlice};

use crate::{
    SharedState,
    atlas::{ImageModifier, ImageWatcher},
    texel::Texel,
};

#[derive(Clone)]
pub struct TextureData {
    state: SharedState,
    extent: wgpu::Extent3d,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

pub struct InnerTexture<U: ImageWatcher>
where
    U::Pixel: Texel,
{
    state: SharedState,
    format: wgpu::TextureFormat,
    dst: Option<TextureData>,
    src: U,
    updates: Rc<RefCell<VecDeque<Rect<u32>>>>,
}

#[derive(Clone)]
pub struct Texture<T: Texel, U: ImageWatcher<Pixel = T>, V: ImageModifier<Pixel = T>>
where
    U::Pixel: Texel,
{
    inner: Rc<RefCell<InnerTexture<U>>>,
    part: V,
}

impl TextureData {
    pub(crate) fn new(state: &SharedState, size: Size2D<u32>, format: wgpu::TextureFormat) -> Self {
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
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = match format.sample_type(None, None) {
            Some(wgpu::TextureSampleType::Uint) => {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &state.uint_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    }],
                    label: None,
                })
            }
            Some(wgpu::TextureSampleType::Float { filterable: true }) => {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &state.float_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&state.float_sampler),
                        },
                    ],
                    label: None,
                })
            }
            _ => panic!("Unsupported texture format: {format:?}"),
        };

        Self {
            state,
            extent,
            texture,
            bind_group,
        }
    }

    fn write<T: Texel>(&self, data: ImageSlice<T>, dst: Point2D<u32>) {
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

impl<U: ImageWatcher> InnerTexture<U>
where
    U::Pixel: Texel,
{
    pub fn new(state: &SharedState, mut image: U, format: wgpu::TextureFormat) -> Self {
        assert!(U::Pixel::is_format_supported(format));
        let mut updates = VecDeque::new();
        updates.push_back(Rect::from_size(image.size()));
        let updates = Rc::new(RefCell::new(updates));
        image.subscribe_to_updates(Rc::downgrade(&updates));
        Self {
            state: state.clone(),
            format,
            dst: None,
            src: image,
            updates,
        }
    }

    fn sync_with_image(&mut self) {
        self.dst.take_if(|texture| {
            Size2D::new(texture.extent.width, texture.extent.height) != self.src.size()
        });
        let dst = match &mut self.dst {
            Some(dst) => dst,
            dst @ None => {
                let texture = TextureData::new(&self.state, self.src.size(), self.format);
                dst.insert(texture)
            }
        };

        for rect in self.updates.borrow_mut().drain(..) {
            self.src
                .with_image_slice(|image| dst.write(image, rect.origin), rect)
        }
    }

    pub fn bind_group_layout(&self) -> wgpu::BindGroupLayout {
        self.state.float_bind_group_layout.clone()
    }
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

impl<U: ImageWatcher> Texture<U> {
    pub fn transform_coord(self, xform: Affine2) -> Self {
        Self {
            xform: xform * self.xform,
            ..self
        }
    }

    pub fn coord_xform(&self) -> Affine2 {
        self.xform
    }
}
