//! A fitted, six-panel layout shared by the drawing galleries.
use crate::Labels;
use wgame::{
    Library,
    gfx::{Camera, Scene, Target},
    glam::{Affine2, Vec2, Vec4},
    prelude::*,
};

pub const SIZE: Vec2 = Vec2::new(1200.0, 900.0);
pub const BACKGROUND: Vec4 = Vec4::new(0.025, 0.035, 0.055, 1.0);

/// Fit the design to a nonempty target and return its physical pixels per unit.
pub fn camera(target: &mut impl Target) -> (Camera, f32) {
    let (width, height) = target.size();
    let viewport = Vec2::new(width as f32, height as f32);
    let scale = (viewport / SIZE).min_element();
    let camera = target
        .physical_camera()
        .transform(Affine2::from_scale_angle_translation(
            Vec2::splat(scale),
            0.0,
            (viewport - scale * SIZE) * 0.5,
        ));
    (camera, scale)
}

pub fn origin(index: usize) -> Vec2 {
    Vec2::new(
        30.0 + (index % 2) as f32 * 600.0,
        110.0 + (index / 2) as f32 * 255.0,
    )
}

pub fn heading(scene: &mut Scene, labels: &mut Labels, title: &str, controls: &str) {
    scene.add(&labels.text(title, 30.0).move_to(Vec2::new(30.0, 42.0)));
    scene.add(&labels.text(controls, 18.0).move_to(Vec2::new(30.0, 77.0)));
}

pub fn panel(
    scene: &mut Scene,
    library: &Library,
    labels: &mut Labels,
    index: usize,
    title: &str,
    detail: &str,
) {
    let origin = origin(index);
    scene.add(
        &library
            .shapes()
            .rectangle((origin, origin + Vec2::new(540.0, 230.0)))
            .fill_color(Vec4::new(0.06, 0.08, 0.11, 1.0)),
    );
    scene.add(
        &labels
            .text(title, 22.0)
            .move_to(origin + Vec2::new(18.0, 30.0)),
    );
    scene.add(
        &labels
            .text(detail, 16.0)
            .move_to(origin + Vec2::new(18.0, 56.0)),
    );
}
