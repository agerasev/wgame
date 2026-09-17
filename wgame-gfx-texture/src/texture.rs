use std::{
    cell::RefCell,
    fmt::{self, Debug},
    rc::Rc,
};

use euclid::default::{Box2D, Point2D, Rect, Size2D, Vector2D};
use glam::{Affine2, Vec2};
use half::f16;
use rgb::Rgba;
use wgame_gfx::types::{Color, color};
use wgame_image::{
    Atlas, AtlasImage, ImageBase, ImageReadExt, ImageSlice, ImageSliceMut, ImageWriteMut,
    atlas::Tracker,
};

use crate::{
    TexturingState,
    gpu::GpuTexture,
    sampling::{
        SampledTexture, TextureAttribute, TextureBinding, TextureRegion, TextureResource,
        TextureSample,
    },
    texel::Texel,
};

pub(crate) struct InnerAtlas<T: Texel> {
    state: TexturingState,
    format: wgpu::TextureFormat,
    dst: Option<GpuTexture>,
    dst_generation: u64,
    src: Atlas<T>,
    tracker: Rc<Tracker>,
}

/// GPU mirror of an append-only [`Atlas`].
///
/// Each CPU atlas generation receives a new GPU texture, even at unchanged
/// dimensions. Existing baked renderers and encoded commands retain the previous
/// GPU texture with matching coordinates. Dead rectangles are reclaimed only by
/// generation replacement; long-lived renderers can retain older GPU textures.
/// See [`Atlas`] for allocation and compaction policy.
#[derive(Clone)]
pub struct TextureAtlas<T: Texel = Rgba<f16>> {
    pub(crate) inner: Rc<RefCell<InnerAtlas<T>>>,
}

/// CPU-editable texture handle tracking its live atlas item.
///
/// Data flows from CPU pixels to the GPU. Sampling is shared with
/// [`crate::RenderTexture`] through [`SampledTexture`], but drawing into this
/// texture is not available:
/// ```compile_fail
/// # fn draw(texture: &mut wgame_gfx_texture::Texture) {
/// use wgame_gfx::Target;
/// texture.clear(wgame_gfx::types::color::BLACK);
/// # }
/// ```
///
/// [`Self::update`] and [`Self::update_part`] maintain the one-pixel border needed
/// for linear filtering; mutating the backing [`AtlasImage`] directly bypasses
/// this maintenance. [`TextureSettings::nearest`] keeps hard texel edges and
/// [`TextureSettings::linear`] interpolates.
///
/// Resizing replaces the item's allocation while clones follow its new location.
/// Existing baked drawing retains its old allocation/texture. Explicit pixel
/// updates remain mutable when synchronized into the same GPU generation.
#[derive(Clone)]
pub struct Texture<T: Texel = Rgba<f16>> {
    atlas: Rc<RefCell<InnerAtlas<T>>>,
    image: AtlasImage<T>,
    settings: TextureSettings,
    xform: Affine2,
    color: Rgba<f32>,
}

pub type FilterMode = wgpu::FilterMode;

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct TextureSettings {
    pub mag_filter: FilterMode,
}

impl TextureSettings {
    pub fn nearest() -> Self {
        Self {
            mag_filter: FilterMode::Nearest,
        }
    }
    pub fn linear() -> Self {
        Self {
            mag_filter: FilterMode::Linear,
        }
    }
}

impl AsRef<Texture> for Texture {
    fn as_ref(&self) -> &Texture {
        self
    }
}

impl<T: Texel> Drop for InnerAtlas<T> {
    fn drop(&mut self) {
        self.src.unsubscribe();
    }
}

impl<T: Texel> InnerAtlas<T> {
    fn new(state: &TexturingState, mut src: Atlas<T>, format: wgpu::TextureFormat) -> Self {
        assert!(T::is_format_supported(format));
        let tracker = Rc::new(Tracker::default());
        src.subscribe(Rc::downgrade(&tracker));
        Self {
            state: state.clone(),
            format,
            dst: None,
            dst_generation: 0,
            src,
            tracker,
        }
    }

    fn sync(&mut self) {
        let generation = self.src.generation();
        if self.dst_generation != generation {
            self.dst = None;
        }
        if let Some(dst) = &self.dst {
            while let Some(rect) = self.tracker.take_next() {
                self.src
                    .with_data(|image| dst.write(image.slice(rect), rect.origin));
            }
        } else {
            let texture = GpuTexture::new(&self.state, self.src.size(), self.format);
            // A replacement needs a complete upload, including a populated atlas
            // first attached to the GPU and same-size compaction generations.
            self.src
                .with_data(|image| texture.write(image.slice((.., ..)), Point2D::zero()));
            self.tracker.clear();
            self.dst = Some(texture);
            self.dst_generation = generation;
        }
    }
}

impl<T: Texel> TextureAtlas<T> {
    pub fn new(state: &TexturingState, src: Atlas<T>, format: wgpu::TextureFormat) -> Self {
        Self {
            inner: Rc::new(RefCell::new(InnerAtlas::new(state, src, format))),
        }
    }

    pub fn state(&self) -> TexturingState {
        self.inner.borrow().state.clone()
    }

    pub fn allocate(&self, size: impl Into<Size2D<u32>>, settings: TextureSettings) -> Texture<T> {
        let size = size.into() + Size2D::new(2, 2);
        let image = self.inner.borrow().src.allocate(size);
        Texture::new(self, image, settings)
    }

    pub fn inner(&self) -> Atlas<T> {
        self.inner.borrow().src.clone()
    }
}

impl<T: Texel> Texture<T> {
    pub fn new(atlas: &TextureAtlas<T>, image: AtlasImage<T>, settings: TextureSettings) -> Self {
        Self {
            atlas: atlas.inner.clone(),
            image,
            settings,
            xform: Affine2::IDENTITY,
            color: color::WHITE.to_rgba(),
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
        assert!(
            rect.origin.x <= size.width
                && rect.origin.y <= size.height
                && rect.size.width <= size.width - rect.origin.x
                && rect.size.height <= size.height - rect.origin.y
        );
        let box_ = rect.to_box2d();
        let inner_rect = Rect {
            origin: rect.origin + Vector2D::new(1, 1),
            size: rect.size,
        };
        if rect.size.width == 0 || rect.size.height == 0 {
            return self.image.update_part(f, inner_rect);
        }
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
                // `img` is relative to the dirty outer rectangle, not the image.
                let local_rect = Rect {
                    origin: inner_rect.origin - outer_box.min.to_vector(),
                    size: rect.size,
                };
                let r = f(img.slice_mut(local_rect));
                let extent = img.size();
                // Duplicate touched edge texels into the one-pixel filtering border.
                // Each source coordinate lies inside this dirty slice.
                let mut copy_border = |x: u32, y: u32| {
                    let global = Point2D::new(outer_box.min.x + x, outer_box.min.y + y);
                    if global.x == 0
                        || global.y == 0
                        || global.x == size.width + 1
                        || global.y == size.height + 1
                    {
                        let source = Point2D::new(
                            global.x.clamp(1, size.width) - outer_box.min.x,
                            global.y.clamp(1, size.height) - outer_box.min.y,
                        );
                        let pixel = *img.get(source);
                        *img.get_mut(Point2D::new(x, y)) = pixel;
                    }
                };
                if extent.width > 0 && extent.height > 0 {
                    for x in 0..extent.width {
                        copy_border(x, 0);
                        copy_border(x, extent.height - 1);
                    }
                    for y in 0..extent.height {
                        copy_border(0, y);
                        copy_border(extent.width - 1, y);
                    }
                }

                r
            },
            outer_box.to_rect(),
        )
    }

    pub fn resize(&self, new_size: impl Into<Size2D<u32>>) {
        let new_size = new_size.into();
        assert!(
            new_size.width > 0 && new_size.height > 0,
            "Texture size must be positive"
        );
        let old = self.with(|src| src.to_image());
        self.image.resize(new_size + Size2D::new(2, 2));
        self.update(|mut dst| {
            for (_, pixel) in dst.pixels_mut() {
                *pixel = T::default();
            }
            let common = Rect::from_size(new_size.min(old.size()));
            dst.slice_mut(common).copy_from(old.slice(common));
        });
    }

    pub fn coord_xform(&self) -> Affine2 {
        TextureRegion::coord_xform(&self.image) * self.xform
    }

    pub fn transform_coord(&self, xform: Affine2) -> Self {
        Self {
            xform: xform * self.xform,
            ..self.clone()
        }
    }

    pub fn multiply_color<C: Color>(&self, color: C) -> Self {
        Self {
            color: self.color.mul(color),
            ..self.clone()
        }
    }

    pub fn resource(&self) -> TextureResource {
        TextureResource::new(self.atlas.clone(), self.settings)
    }

    pub fn attribute(&self) -> TextureAttribute {
        self.sample().attribute()
    }
}

impl<T: Texel> TextureBinding for RefCell<InnerAtlas<T>> {
    fn bind_group(&self, settings: TextureSettings) -> wgpu::BindGroup {
        let mut atlas = self.borrow_mut();
        atlas.sync();
        atlas.dst.as_ref().unwrap().bind_group(settings)
    }
    fn bind_group_layout(&self) -> wgpu::BindGroupLayout {
        let atlas = self.borrow();
        atlas.state.bind_group_layout(atlas.format)
    }
}

impl<T: Texel> TextureRegion for AtlasImage<T> {
    fn size(&self) -> Size2D<u32> {
        self.size() - Size2D::new(2, 2)
    }
    fn coord_xform(&self) -> Affine2 {
        let atlas_size = self.atlas().size();
        let rect = self.rect();
        Affine2::from_translation(Vec2::new(
            (rect.origin.x + 1) as f32 / atlas_size.width as f32,
            (rect.origin.y + 1) as f32 / atlas_size.height as f32,
        )) * Affine2::from_scale(Vec2::new(
            rect.size.width.saturating_sub(2) as f32 / atlas_size.width as f32,
            rect.size.height.saturating_sub(2) as f32 / atlas_size.height as f32,
        ))
    }
}

impl<T: Texel> SampledTexture for Texture<T> {
    fn sample(&self) -> TextureSample {
        TextureSample::new(self.resource(), Rc::new(self.image.clone()))
            .transform_coord(self.xform)
            .multiply_color(self.color)
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
