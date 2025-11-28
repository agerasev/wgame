use std::{
    cell::{RefCell, RefMut},
    cmp::Ordering,
    collections::vec_deque::VecDeque,
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    ops::Deref,
    rc::Rc,
};

use euclid::default::{Box2D, Point2D, Rect, Size2D, Vector2D};
use glam::{Affine2, Vec2};
use half::f16;
use rgb::Rgba;
use wgame_image::{
    Atlas, AtlasImage, ImageBase, ImageRead, ImageReadExt, ImageSlice, ImageSliceMut,
    ImageWriteMut, atlas::Tracker,
};
use wgame_shader::{Attribute, BindingList, BytesSink};

use crate::{TextureState, texel::Texel};

#[derive(Clone)]
struct TextureInstance {
    state: TextureState,
    extent: wgpu::Extent3d,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

pub(crate) struct InnerAtlas<T: Texel> {
    state: TextureState,
    format: wgpu::TextureFormat,
    dst: Option<TextureInstance>,
    src: Atlas<T>,
    tracker: Rc<Tracker>,
}

#[derive(Clone)]
pub struct TextureAtlas<T: Texel = Rgba<f16>> {
    pub(crate) inner: Rc<RefCell<InnerAtlas<T>>>,
}

#[derive(Clone)]
pub struct Texture<T: Texel = Rgba<f16>> {
    atlas: Rc<RefCell<InnerAtlas<T>>>,
    image: AtlasImage<T>,
    xform: Affine2,
}

impl TextureInstance {
    fn new(state: &TextureState, size: Size2D<u32>, format: wgpu::TextureFormat) -> Self {
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

impl<T: Texel> Drop for InnerAtlas<T> {
    fn drop(&mut self) {
        self.src.unsubscribe();
    }
}

impl<T: Texel> InnerAtlas<T> {
    fn new(state: &TextureState, mut src: Atlas<T>, format: wgpu::TextureFormat) -> Self {
        assert!(T::is_format_supported(format));
        let mut updates = VecDeque::new();
        updates.push_back(Rect::from_size(src.size()));
        let tracker = Rc::new(Tracker::default());
        src.subscribe(Rc::downgrade(&tracker));
        Self {
            state: state.clone(),
            format,
            dst: None,
            src,
            tracker,
        }
    }

    fn sync(&mut self) -> TextureInstance {
        self.dst.take_if(|texture| {
            Size2D::new(texture.extent.width, texture.extent.height) != self.src.size()
        });
        let dst = match &mut self.dst {
            Some(dst) => dst,
            dst @ None => {
                let texture = TextureInstance::new(&self.state, self.src.size(), self.format);
                dst.insert(texture)
            }
        };

        while let Some(rect) = self.tracker.take_next() {
            self.src
                .with_data(|image| dst.write(image.slice(rect), rect.origin))
        }

        dst.clone()
    }
}

impl<T: Texel> TextureAtlas<T> {
    pub fn new(state: &TextureState, src: Atlas<T>, format: wgpu::TextureFormat) -> Self {
        Self {
            inner: Rc::new(RefCell::new(InnerAtlas::new(state, src, format))),
        }
    }

    pub fn state(&self) -> TextureState {
        self.inner.borrow().state.clone()
    }

    pub fn allocate(&self, size: impl Into<Size2D<u32>>) -> Texture<T> {
        let size = size.into() + Size2D::new(2, 2);
        let image = self.inner.borrow().src.allocate(size);
        Texture::new(&self, image)
    }

    pub fn inner(&self) -> Atlas<T> {
        self.inner.borrow().src.clone()
    }
}

impl<T: Texel> Texture<T> {
    pub fn new(atlas: &TextureAtlas<T>, image: AtlasImage<T>) -> Self {
        Self {
            atlas: atlas.inner.clone(),
            image,
            xform: Affine2::IDENTITY,
        }
    }

    pub fn atlas(&self) -> TextureAtlas<T> {
        TextureAtlas {
            inner: self.atlas.clone(),
        }
    }

    pub fn image(&self) -> &AtlasImage<T> {
        &self.image
    }

    pub fn size(&self) -> Size2D<u32> {
        self.image.size() - Size2D::new(2, 2)
    }

    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(ImageSlice<T>) -> R,
    {
        self.image.with(|img| {
            let rect = Rect {
                origin: Point2D::new(1, 1),
                size: img.size() - Size2D::new(2, 2),
            };
            f(img.slice(rect))
        })
    }

    pub fn update<F, R>(&self, f: F) -> R
    where
        F: FnOnce(ImageSliceMut<T>) -> R,
    {
        self.update_part(f, Rect::from_size(self.size()))
    }

    pub fn update_part<F, R>(&self, f: F, rect: Rect<u32>) -> R
    where
        F: FnOnce(ImageSliceMut<T>) -> R,
    {
        let size = self.size();
        let box_ = rect.to_box2d();
        assert!(box_.max.x <= size.width && box_.max.y <= size.height);
        let inner_rect = Rect {
            origin: rect.origin + Vector2D::new(1, 1),
            size: rect.size,
        };
        let outer_box = Box2D {
            min: Point2D::new(
                if box_.min.x < 1 { 0 } else { box_.min.x + 1 },
                if box_.min.y < 1 { 0 } else { box_.min.y + 1 },
            ),
            max: Point2D::new(
                if box_.max.x >= size.width {
                    size.width + 2
                } else {
                    box_.max.x + 1
                },
                if box_.max.y >= size.height {
                    size.height + 2
                } else {
                    box_.max.y + 1
                },
            ),
        };

        self.image.update_part(
            |mut img| {
                // Update inner image
                let r = f(img.slice_mut(inner_rect));

                // Update borders if needed
                if box_.min.x < 1 {
                    img.copy_within(
                        Rect {
                            origin: Point2D::new(1, 1),
                            size: Size2D::new(1, rect.size.height),
                        },
                        Point2D::new(0, 1),
                    );
                }
                if box_.min.y < 1 {
                    img.copy_within(
                        Rect {
                            origin: Point2D::new(1, 1),
                            size: Size2D::new(rect.size.width, 1),
                        },
                        Point2D::new(1, 0),
                    );
                }
                if box_.max.x >= size.width {
                    img.copy_within(
                        Rect {
                            origin: Point2D::new(rect.size.width, 1),
                            size: Size2D::new(1, rect.size.height),
                        },
                        Point2D::new(rect.size.width + 1, 1),
                    );
                }
                if box_.max.y >= size.height {
                    img.copy_within(
                        Rect {
                            origin: Point2D::new(1, rect.size.height),
                            size: Size2D::new(rect.size.width, 1),
                        },
                        Point2D::new(1, rect.size.height + 1),
                    );
                }
                // Update corners is needed
                if box_.min.x < 1 && box_.min.y < 1 {
                    *img.get_mut(Point2D::new(0, 0)) = *img.get(Point2D::new(1, 1));
                }
                if box_.min.x < 1 && box_.max.y >= size.height {
                    *img.get_mut(Point2D::new(0, rect.size.height + 1)) =
                        *img.get(Point2D::new(1, rect.size.height));
                }
                if box_.max.x >= size.width && box_.min.y < 1 {
                    *img.get_mut(Point2D::new(rect.size.width + 1, 0)) =
                        *img.get(Point2D::new(rect.size.width, 1));
                }
                if box_.max.x >= size.width && box_.max.y >= size.height {
                    *img.get_mut(Point2D::new(rect.size.width + 1, rect.size.height + 1)) =
                        *img.get(Point2D::new(rect.size.width, rect.size.height));
                }

                r
            },
            outer_box.to_rect(),
        )
    }

    pub fn resize(&self, new_size: impl Into<Size2D<u32>>) {
        self.image.resize(new_size.into() + Size2D::new(2, 2));
    }

    pub fn coord_xform(&self) -> Affine2 {
        let atlas_size = self.atlas.borrow().src.size();
        let Rect { origin, size } = self.image.rect();
        let item_rect = Rect {
            origin: Point2D::new(origin.x + 1, origin.y + 1),
            size: Size2D::new(size.width.saturating_sub(2), size.width.saturating_sub(2)),
        };
        let item_xform = Affine2::from_translation(Vec2::new(
            item_rect.origin.x as f32 / atlas_size.width as f32,
            item_rect.origin.y as f32 / atlas_size.height as f32,
        )) * Affine2::from_scale(Vec2::new(
            item_rect.size.width as f32 / atlas_size.width as f32,
            item_rect.size.height as f32 / atlas_size.height as f32,
        ));
        item_xform * self.xform
    }

    pub fn transform_coord(self, xform: Affine2) -> Self {
        Self {
            xform: xform * self.xform,
            ..self
        }
    }

    pub fn resource(&self) -> TextureResource<T> {
        TextureResource {
            atlas: self.atlas.clone(),
        }
    }

    pub fn attribute(&self) -> TextureAttribute<T> {
        TextureAttribute(self.clone())
    }
}

impl<T: Texel> Deref for Texture<T> {
    type Target = AtlasImage<T>;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}

#[derive(Clone)]
pub struct TextureResource<T: Texel = Rgba<f16>> {
    atlas: Rc<RefCell<InnerAtlas<T>>>,
}

impl<T: Texel> TextureResource<T> {
    fn get_instance(&self) -> RefMut<'_, TextureInstance> {
        let mut atlas = self.atlas.borrow_mut();
        atlas.sync();
        RefMut::map(atlas, |atlas| atlas.dst.as_mut().unwrap())
    }

    pub fn bind_group(&self) -> wgpu::BindGroup {
        self.get_instance().bind_group.clone()
    }

    pub fn bind_group_layout(&self) -> wgpu::BindGroupLayout {
        let instance = self.get_instance();
        let format = instance.texture.format();
        instance.state.bind_group_layout(format)
    }
}

impl<T: Texel> PartialOrd for TextureResource<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T: Texel> Ord for TextureResource<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.atlas.as_ptr().cmp(&other.atlas.as_ptr())
    }
}

impl<T: Texel> PartialEq for TextureResource<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.atlas, &other.atlas)
    }
}
impl<T: Texel> Eq for TextureResource<T> {}

impl<T: Texel> Hash for TextureResource<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.atlas.as_ptr().hash(state);
    }
}

impl<T: Texel> Debug for TextureResource<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Texture atlas at {:?}", self.atlas.as_ptr())
    }
}

#[derive(Clone)]
pub struct TextureAttribute<T: Texel = Rgba<f16>>(Texture<T>);

impl<T: Texel> Attribute for TextureAttribute<T> {
    fn bindings() -> BindingList {
        <glam::Affine2 as Attribute>::bindings()
    }

    const SIZE: usize = <glam::Affine2 as Attribute>::SIZE;

    fn store(&self, dst: &mut BytesSink) {
        self.0.coord_xform().store(dst);
    }
}

impl<T: Texel> Debug for Texture<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct Texture {
            pub atlas: *mut (),
            pub image: Rect<u32>,
            pub xform: Affine2,
        }

        Texture {
            atlas: self.atlas.as_ptr() as *mut (),
            image: self.image.rect(),
            xform: self.xform,
        }
        .fmt(f)
    }
}

impl<T: Texel> Debug for TextureAtlas<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TextureAtlas ({:?})", self.inner.as_ptr() as *mut ())
    }
}
