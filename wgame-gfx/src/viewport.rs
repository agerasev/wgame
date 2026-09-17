use std::{error::Error, fmt};

use glam::Vec2;

use crate::{Graphics, Target};

/// A borrowed rectangle of a render target, created by [`Target::viewport`].
///
/// Coordinates passed at construction are physical pixels relative to the parent.
/// [`Target::size`] and cameras use this rectangle's local coordinates; drawing
/// and clearing are clipped to it. Nested viewports share the same color/depth
/// attachments and encoder. Creating or dropping a viewport does not clear,
/// allocate a texture, submit commands, or present a frame. Finish its borrow
/// before using the parent again.
///
/// This is a graphics view, not an input or presentation host. For picking, convert
/// a pointer to attachment pixels (including the host's DPI scale), then use
/// [`Self::to_local`] before [`crate::Camera::screen_to_world`] or
/// [`crate::Camera::screen_ray`]. Raw encoder/attachment access remains unrestricted.
///
/// ```no_run
/// use wgame_gfx::{Target, types::color};
/// # fn preview(target: &mut impl Target) -> Result<(), wgame_gfx::ViewportError> {
/// {
///     let mut preview = target.viewport((20, 30), (160, 90))?;
///     preview.clear(color::BLACK); // only this rectangle, including its depth
///     let camera = preview.physical_camera(); // (0, 0)..(160, 90)
///     let mut scene = preview.scene();
///     scene.camera = camera;
///     // Add ordinary 2D/3D objects, or use preview.render(&camera, &baked_scene).
///     scene.render();
/// }
/// // The parent can now render or present normally.
/// # Ok(()) }
/// ```
pub struct Viewport<'a, T: Target + ?Sized> {
    parent: &'a mut T,
    region: Region,
}

impl<'a, T: Target + ?Sized> Viewport<'a, T> {
    pub(crate) fn new(
        parent: &'a mut T,
        origin: (u32, u32),
        size: (u32, u32),
    ) -> Result<Self, ViewportError> {
        let region = Region {
            origin: parent.origin(),
            size: parent.size(),
        }
        .child(origin, size)?;
        Ok(Self { parent, region })
    }

    /// Convert attachment-relative physical pixels to local physical pixels.
    /// Points outside the rectangle are allowed, e.g. during a captured drag.
    /// Nested viewport offsets are included automatically.
    pub fn to_local(&self, position: Vec2) -> Vec2 {
        position - Vec2::new(self.region.origin.0 as f32, self.region.origin.1 as f32)
    }
}

impl<T: Target + ?Sized> Target for Viewport<'_, T> {
    fn state(&self) -> &Graphics {
        self.parent.state()
    }
    fn view(&self) -> &wgpu::TextureView {
        self.parent.view()
    }
    fn depth_view(&self) -> &wgpu::TextureView {
        self.parent.depth_view()
    }
    fn encoder(&mut self) -> &mut wgpu::CommandEncoder {
        self.parent.encoder()
    }
    fn origin(&self) -> (u32, u32) {
        self.region.origin
    }
    fn size(&self) -> (u32, u32) {
        self.region.size
    }
    fn premultiplied_alpha(&self) -> bool {
        self.parent.premultiplied_alpha()
    }
}

/// Invalid viewport rectangle. The parent remains usable after an error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewportError {
    /// Width or height is zero. Skip hidden/empty drawing areas before rendering.
    Empty,
    /// The rectangle extends beyond its parent or its coordinates overflow.
    OutOfBounds,
}
impl fmt::Display for ViewportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "viewport width and height must be positive",
            Self::OutOfBounds => "viewport must fit inside its parent target",
        })
    }
}
impl Error for ViewportError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Region {
    pub origin: (u32, u32),
    pub size: (u32, u32),
}
impl Region {
    fn child(self, origin: (u32, u32), size: (u32, u32)) -> Result<Self, ViewportError> {
        if size.0 == 0 || size.1 == 0 {
            return Err(ViewportError::Empty);
        }
        let fits = |offset: u32, extent: u32, parent: u32| {
            offset.checked_add(extent).is_some_and(|end| end <= parent)
        };
        if !fits(origin.0, size.0, self.size.0) || !fits(origin.1, size.1, self.size.1) {
            return Err(ViewportError::OutOfBounds);
        }
        let x = self
            .origin
            .0
            .checked_add(origin.0)
            .ok_or(ViewportError::OutOfBounds)?;
        let y = self
            .origin
            .1
            .checked_add(origin.1)
            .ok_or(ViewportError::OutOfBounds)?;
        Ok(Self {
            origin: (x, y),
            size,
        })
    }
    pub fn apply(self, pass: &mut wgpu::RenderPass<'_>) {
        let (x, y) = self.origin;
        let (width, height) = self.size;
        pass.set_viewport(x as f32, y as f32, width as f32, height as f32, 0.0, 1.0);
        pass.set_scissor_rect(x, y, width, height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_rectangles_are_parent_relative_and_may_touch_edges() {
        let root = Region {
            origin: (0, 0),
            size: (100, 80),
        };
        assert_eq!(root.child((0, 0), root.size).unwrap(), root);
        let parent = root.child((20, 10), (60, 50)).unwrap();
        assert_eq!(
            parent.child((10, 20), (50, 30)).unwrap(),
            Region {
                origin: (30, 30),
                size: (50, 30)
            }
        );
        assert_eq!(parent.child((59, 49), (1, 1)).unwrap().origin, (79, 59));
    }
    #[test]
    fn empty_outside_and_overflowing_rectangles_are_rejected() {
        let root = Region {
            origin: (0, 0),
            size: (100, 80),
        };
        for size in [(0, 1), (1, 0)] {
            assert_eq!(root.child((0, 0), size), Err(ViewportError::Empty));
        }
        for (origin, size) in [
            ((100, 0), (1, 1)),
            ((0, 80), (1, 1)),
            ((90, 70), (11, 10)),
            ((u32::MAX, 0), (1, 1)),
            ((1, 0), (u32::MAX, 1)),
        ] {
            assert_eq!(root.child(origin, size), Err(ViewportError::OutOfBounds));
        }
    }
}
