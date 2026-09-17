//! Sampling shared by CPU atlas items and GPU render targets.
use crate::TextureSettings;
use euclid::default::Size2D;
use glam::{Affine2, Vec4};
use rgb::Rgba;
use std::{
    fmt,
    hash::{Hash, Hasher},
    rc::Rc,
};
use wgame_gfx::types::{Color, color};
use wgame_shader::{Attribute, BindingList, BytesSink};

/// A texture usable by drawing APIs, independent of how its pixels are produced.
///
/// [`crate::Texture`] supplies CPU uploads; [`crate::RenderTexture`] supplies GPU
/// rendering. The returned handle owns its resources and carries no pixel editing
/// or rendering capability. Wrappers can implement this trait by returning a
/// transformed sample of either texture. The sampled format must match the
/// renderer's binding layout (built-in shapes use filterable float textures).
pub trait SampledTexture {
    fn sample(&self) -> TextureSample;
}

pub(crate) trait TextureBinding {
    fn bind_group(&self, settings: TextureSettings) -> wgpu::BindGroup;
    fn bind_group_layout(&self) -> wgpu::BindGroupLayout;
}
pub(crate) trait TextureRegion {
    fn size(&self) -> Size2D<u32>;
    fn coord_xform(&self) -> Affine2;
}

/// Owned sampling resource. Equality identifies a shared source and sampler,
/// allowing different items in one atlas to batch. Atlas uploads and generation
/// resolution happen when a bind group is requested, preserving baked resources.
#[derive(Clone)]
pub struct TextureResource {
    source: Rc<dyn TextureBinding>,
    settings: TextureSettings,
}
impl TextureResource {
    pub(crate) fn new(source: Rc<dyn TextureBinding>, settings: TextureSettings) -> Self {
        Self { source, settings }
    }
    pub fn bind_group(&self) -> wgpu::BindGroup {
        self.source.bind_group(self.settings)
    }
    pub fn bind_group_layout(&self) -> wgpu::BindGroupLayout {
        self.source.bind_group_layout()
    }
}
impl PartialEq for TextureResource {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.source, &other.source) && self.settings == other.settings
    }
}
impl Eq for TextureResource {}
impl Hash for TextureResource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.source).hash(state);
        self.settings.hash(state);
    }
}
impl fmt::Debug for TextureResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextureResource")
            .field("source", &Rc::as_ptr(&self.source))
            .field("settings", &self.settings)
            .finish()
    }
}

/// Read-only sampling handle retained by shapes and scenes.
///
/// CPU atlas items retain live placement until baking. Render textures retain
/// their GPU allocation; subsequent submitted rendering changes its pixels even
/// in baked scenes. Dropping the original texture does not invalidate a sample.
#[derive(Clone)]
pub struct TextureSample {
    resource: TextureResource,
    region: Rc<dyn TextureRegion>,
    xform: Affine2,
    color: Rgba<f32>,
}
impl TextureSample {
    pub(crate) fn new(resource: TextureResource, region: Rc<dyn TextureRegion>) -> Self {
        Self {
            resource,
            region,
            xform: Affine2::IDENTITY,
            color: color::WHITE.to_rgba(),
        }
    }
    pub fn size(&self) -> Size2D<u32> {
        self.region.size()
    }
    pub fn coord_xform(&self) -> Affine2 {
        self.region.coord_xform() * self.xform
    }
    pub fn resource(&self) -> TextureResource {
        self.resource.clone()
    }
    pub fn attribute(&self) -> TextureAttribute {
        TextureAttribute {
            texture: self.clone(),
            local_xform: Affine2::IDENTITY,
        }
    }
    pub fn with_settings(&self, settings: TextureSettings) -> Self {
        Self {
            resource: TextureResource {
                settings,
                ..self.resource.clone()
            },
            ..self.clone()
        }
    }
    pub fn transform_coord(&self, xform: Affine2) -> Self {
        Self {
            xform: xform * self.xform,
            ..self.clone()
        }
    }
    pub fn multiply_color(&self, color: impl Color) -> Self {
        Self {
            color: self.color.mul(color),
            ..self.clone()
        }
    }
}
impl SampledTexture for TextureSample {
    fn sample(&self) -> TextureSample {
        self.clone()
    }
}
impl fmt::Debug for TextureSample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextureSample")
            .field("resource", &self.resource)
            .field("size", &self.size())
            .field("xform", &self.xform)
            .finish()
    }
}

/// Per-instance UV mapping and tint, shared by every sampling source.
#[derive(Clone, Debug)]
pub struct TextureAttribute {
    texture: TextureSample,
    local_xform: Affine2,
}
impl TextureAttribute {
    pub fn coord_xform(&self) -> Affine2 {
        self.texture.coord_xform() * self.local_xform
    }
    pub fn color(&self) -> Rgba<f32> {
        self.texture.color
    }

    /// Map primitive coordinates before the texture transform and atlas placement.
    /// Repeated calls compose as `previous * mapping`. Atlas placement remains
    /// live until serialization; ribbon segments can select separate UV intervals.
    pub fn map_coord(&self, mapping: Affine2) -> Self {
        Self {
            local_xform: self.local_xform * mapping,
            ..self.clone()
        }
    }
}
impl Attribute for TextureAttribute {
    fn bindings() -> BindingList {
        BindingList::chain(
            <Affine2 as Attribute>::bindings().with_prefix("xform"),
            <Vec4 as Attribute>::bindings().with_prefix("color"),
        )
    }
    const SIZE: usize = <Affine2 as Attribute>::SIZE + <Vec4 as Attribute>::SIZE;
    fn store(&self, dst: &mut BytesSink) {
        self.coord_xform().store(dst);
        self.color().to_vec4().store(dst);
    }
}
