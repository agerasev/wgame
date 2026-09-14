#[path = "../../tests/support/mod.rs"]
mod support;

#[cfg(feature = "shapes")]
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn logical_camera_scales_pixels_and_unprojects_input() {
    use crate::{
        Library,
        gfx::{Offscreen, Scene, types::color},
        prelude::*,
    };
    use glam::Vec2;
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    for factor in [1.0, 1.25, 2.0] {
        let size = ((80.0 * factor) as u32, (64.0 * factor) as u32);
        let mut target = Offscreen::new(&gfx, size);
        let camera = super::logical_camera(&mut target, factor);
        let point = Vec2::new(24.0, 32.0);
        let actual = camera.screen_to_world(point * factor as f32, size).unwrap();
        assert!((actual - point).length() < 0.001);
        let mut scene = Scene::default();
        scene.add(
            &lib.shapes()
                .rectangle((Vec2::new(8.0, 16.0), Vec2::new(40.0, 48.0)))
                .fill_color(color::WHITE),
        );
        target.clear(color::BLACK);
        target.render_iter(&camera, scene.iter());
        let pixels = support::pixels(&mut target);
        for y in 0..size.1 {
            for x in 0..size.0 {
                let inside = (8.0 * factor..40.0 * factor).contains(&(x as f64))
                    && (16.0 * factor..48.0 * factor).contains(&(y as f64));
                let expected = if inside {
                    [255, 255, 255, 255]
                } else {
                    [0, 0, 0, 255]
                };
                let offset = ((y * size.0 + x) * 4) as usize;
                assert_eq!(
                    &pixels[offset..offset + 4],
                    &expected,
                    "scale {factor}, pixel {x},{y}"
                );
            }
        }
    }
}

#[test]
fn window_size_configuration_preserves_units() {
    use crate::WindowConfig;
    use winit::dpi::{LogicalSize, PhysicalSize, Size};
    assert_eq!(
        WindowConfig::default().size((801, 601)).app.inner_size,
        Some(Size::Physical(PhysicalSize::new(801, 601)))
    );
    assert_eq!(
        WindowConfig::default()
            .logical_size((801.5, 601.5))
            .app
            .inner_size,
        Some(Size::Logical(LogicalSize::new(801.5, 601.5)))
    );
}

#[cfg(feature = "typography")]
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn logical_text_matches_native_resolution_raster() {
    use crate::{
        Library,
        gfx::{Offscreen, Scene, types::color},
        prelude::*,
    };
    use glam::Vec2;
    let gfx = support::graphics();
    let library = Library::new(&gfx);
    let data = crate::typography::FontData::new(
        include_bytes!("../../../wgame-examples/assets/free-sans-bold.ttf").to_vec(),
        0,
    )
    .unwrap();
    let font = library.make_font(&data);
    for factor in [1.0, 1.25, 2.0] {
        let mut target = Offscreen::new(&gfx, (320, 100));
        let raster = font.rasterize(18.0 * factor as f32);
        let text = raster.text("HiDPI text");
        // Keep the baseline on physical pixels at every factor: nearest glyph
        // sampling at half-pixel ties is sensitive to matrix rounding.
        let camera = super::logical_camera(&mut target, factor);
        let mut scene = Scene::default();
        scene.add(&text.scale(18.0).move_to(Vec2::new(8.0, 32.0)));
        target.clear(color::BLACK);
        target.render_iter(&camera, scene.iter());
        let logical = support::pixels(&mut target);
        let camera = target.physical_camera();
        let mut scene = Scene::default();
        scene.add(
            &text
                .scale(18.0 * factor as f32)
                .move_to(Vec2::new(8.0, 32.0) * factor as f32),
        );
        target.clear(color::BLACK);
        target.render_iter(&camera, scene.iter());
        let physical = support::pixels(&mut target);
        assert!(logical.as_chunks::<4>().0.iter().any(|p| p[0] > 128));
        let differences: Vec<_> = logical
            .iter()
            .zip(&physical)
            .map(|(a, b)| a.abs_diff(*b))
            .collect();
        assert!(
            differences.iter().all(|d| *d <= 1),
            "factor {factor}: max {:?}, sum {}, changed {}",
            differences.iter().max(),
            differences.iter().map(|d| *d as u64).sum::<u64>(),
            differences.iter().filter(|d| **d > 1).count()
        );
    }
}
