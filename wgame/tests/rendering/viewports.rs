use super::*;
use wgame::{
    gfx::{Camera, ViewportError},
    glam::{Mat4, Vec3},
};

fn at(data: &[u8], width: usize, x: usize, y: usize) -> [u8; 4] {
    data[(y * width + x) * 4..][..4].try_into().unwrap()
}
fn inside(x: usize, y: usize, origin: (usize, usize), size: (usize, usize)) -> bool {
    x >= origin.0 && x < origin.0 + size.0 && y >= origin.1 && y < origin.1 + size.1
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn nested_clears_preserve_neighbors_and_parent_rendering() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let mut target = Offscreen::new(&gfx, (96, 80));
    target.clear(color::BLUE);
    {
        let mut outer = target.viewport((10, 8), (70, 60)).unwrap();
        assert_eq!(outer.size(), (70, 60));
        assert_eq!(outer.origin(), (10, 8));
        outer.clear(color::RED);
        {
            let mut inner = outer.viewport((5, 7), (20, 16)).unwrap();
            assert_eq!(inner.origin(), (15, 15));
            let pointer = inner.to_local(Vec2::new(18.0, 20.0));
            assert_eq!(pointer, Vec2::new(3.0, 5.0));
            let camera = inner.physical_camera();
            let picked = camera.screen_to_world(pointer, inner.size()).unwrap();
            assert!(picked.distance(pointer) < 0.001);
            inner.clear(color::GREEN);
        }
        // Nonopaque clear replaces RGBA instead of blending with earlier color.
        outer
            .viewport((45, 35), (10, 10))
            .unwrap()
            .clear(Rgba::new(0.2, 0.4, 0.6, 0.0));
        let camera = outer.physical_camera();
        let mut scene = outer.scene();
        scene.camera = camera;
        scene.add(
            &lib.shapes()
                .rectangle((Vec2::new(40.0, 2.0), Vec2::new(50.0, 8.0)))
                .fill_color(color::YELLOW),
        );
        // AutoScene must obey the borrowed target as well.
    }
    let camera = target.physical_camera();
    let mut scene = Scene::default();
    scene.add(
        &lib.shapes()
            .rectangle((Vec2::ZERO, Vec2::splat(6.0)))
            .fill_color(color::MAGENTA),
    );
    target.render(&camera, &scene.bake());
    // All differently colored clears are encoded before this single readback.
    let data = support::pixels(&mut target);
    for y in 0..80 {
        for x in 0..96 {
            let expected = if inside(x, y, (0, 0), (6, 6)) {
                [255, 0, 255, 255]
            } else if inside(x, y, (50, 10), (10, 6)) {
                [255, 255, 0, 255]
            } else if inside(x, y, (55, 43), (10, 10)) {
                [51, 102, 153, 0]
            } else if inside(x, y, (15, 15), (20, 16)) {
                [0, 255, 0, 255]
            } else if inside(x, y, (10, 8), (70, 60)) {
                [255, 0, 0, 255]
            } else {
                [0, 0, 255, 255]
            };
            assert_eq!(at(&data, 96, x, y), expected, "at {x},{y}");
        }
    }
    // A viewport never submits on drop; discarding its parent discards its draws.
    target
        .viewport((10, 8), (70, 60))
        .unwrap()
        .clear(color::WHITE);
    target.discard();
    assert_eq!(support::pixels(&mut target), data);
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn drawing_is_clipped_and_camera_coordinates_are_local_after_resize() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    for size in [(64, 48), (96, 80)] {
        let mut target = Offscreen::new(&gfx, size);
        target.clear(color::BLACK);
        {
            let mut viewport = target.viewport((16, 12), (20, 10)).unwrap();
            let camera = viewport.physical_camera();
            let mut scene = Scene::default();
            // Intentionally much larger than the viewport.
            scene.add(
                &lib.shapes()
                    .rectangle((Vec2::splat(-100.0), Vec2::splat(100.0)))
                    .fill_color(color::WHITE),
            );
            scene.add(
                &lib.shapes()
                    .rectangle((Vec2::new(3.0, 2.0), Vec2::new(7.0, 5.0)))
                    .fill_color(color::GREEN),
            );
            viewport.render_iter(&camera, scene.iter());
        }
        // A failed borrow leaves the parent usable and does not emit commands.
        assert!(matches!(
            target.viewport((0, 0), (0, 1)),
            Err(ViewportError::Empty)
        ));
        assert!(matches!(
            target.viewport((size.0, 0), (1, 1)),
            Err(ViewportError::OutOfBounds)
        ));
        let data = support::pixels(&mut target);
        for y in 0..size.1 as usize {
            for x in 0..size.0 as usize {
                let expected = if inside(x, y, (19, 14), (4, 3)) {
                    [0, 255, 0, 255]
                } else if inside(x, y, (16, 12), (20, 10)) {
                    [255; 4]
                } else {
                    [0, 0, 0, 255]
                };
                assert_eq!(at(&data, size.0 as usize, x, y), expected, "at {x},{y}");
            }
        }
    }
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn partial_depth_and_color_clears_reset_only_their_region() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let mut target = Offscreen::new(&gfx, (64, 48));
    let camera = Camera::new(&gfx, Mat4::IDENTITY);
    let quad = lib.shapes().unit_quad();
    let render = |target: &mut Offscreen, color, depth| {
        let mut scene = Scene::default();
        scene.add(&quad.fill_color(color).move_to(Vec3::Z * depth));
        target.render(&camera, &scene.bake());
    };
    target.clear(color::BLACK);
    render(&mut target, color::RED, 0.2);
    let before = support::pixels(&mut target);
    target.viewport((8, 6), (24, 18)).unwrap().clear_depth();
    assert_eq!(
        before,
        support::pixels(&mut target),
        "depth clear changed color"
    );
    render(&mut target, color::BLUE, 0.8);
    let data = support::pixels(&mut target);
    assert_eq!(at(&data, 64, 10, 10), [0, 0, 255, 255]);
    assert_eq!(at(&data, 64, 40, 30), [255, 0, 0, 255]);
    target
        .viewport((8, 6), (24, 18))
        .unwrap()
        .clear(color::GREEN);
    render(&mut target, color::CYAN, 0.9);
    let data = support::pixels(&mut target);
    for y in 0..48 {
        for x in 0..64 {
            let expected = if inside(x, y, (8, 6), (24, 18)) {
                [0, 255, 255, 255]
            } else {
                [255, 0, 0, 255]
            };
            assert_eq!(at(&data, 64, x, y), expected);
        }
    }
    // A viewport covering the attachment also works through the fast clear path.
    target
        .viewport((0, 0), (64, 48))
        .unwrap()
        .clear(color::GREEN);
    assert!(
        support::pixels(&mut target)
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [0, 255, 0, 255])
    );
}
