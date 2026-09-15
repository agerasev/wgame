use super::*;
use wgame::{
    gfx::{Camera, DepthMode},
    glam::{Mat4, Vec3},
};

fn center(target: &mut Offscreen) -> [u8; 4] {
    let pixels = support::pixels(target);
    let width = target.size().0 as usize;
    let height = target.size().1 as usize;
    pixels[(height / 2 * width + width / 2) * 4..][..4]
        .try_into()
        .unwrap()
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn depth_orders_geometry_across_passes_and_equal_depth_keeps_order() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    for size in [(32, 32), (64, 48)] {
        let mut target = Offscreen::new(&gfx, size);
        let camera = Camera::new(&gfx, Mat4::IDENTITY);
        let quad = lib.shapes().unit_quad();
        let red = quad.fill_color(color::RED).move_to(Vec3::Z * 0.2);
        let blue = quad.fill_color(color::BLUE).move_to(Vec3::Z * 0.8);
        target.clear(color::BLACK);
        for object in [&red, &blue] {
            let mut scene = Scene::default();
            scene.add(object);
            target.render(&camera, &scene.bake());
        }
        assert_eq!(center(&mut target), [255, 0, 0, 255]);
        let mut equal = Scene::default();
        equal.add(&quad.fill_color(color::GREEN).move_to(Vec3::Z * 0.2));
        target.render_iter(&camera, equal.iter());
        assert_eq!(center(&mut target), [0, 255, 0, 255]);
        target.clear_depth();
        let mut farther = Scene::default();
        farther.add(&blue);
        target.render_iter(&camera, farther.iter());
        assert_eq!(center(&mut target), [0, 0, 255, 255]);
        target.clear(color::BLACK);
        target.render_iter(&camera, farther.iter());
        assert_eq!(center(&mut target), [0, 0, 255, 255]);
    }
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn transparent_holes_read_only_blending_and_overlay_have_explicit_depth_policy() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let mut target = Offscreen::new(&gfx, (32, 32));
    let camera = Camera::new(&gfx, Mat4::IDENTITY);
    let quad = lib.shapes().unit_quad();
    let mut scene = Scene::default();
    scene.add(
        &quad
            .fill_color(Rgba::new(1.0, 1.0, 1.0, 0.0))
            .move_to(Vec3::Z * 0.1),
    );
    scene.add(&quad.fill_color(color::BLUE).move_to(Vec3::Z * 0.8));
    scene.add(
        &quad
            .fill_color(Rgba::new(1.0, 0.0, 0.0, 0.5))
            .depth(DepthMode::ReadOnly)
            .move_to(Vec3::Z * 0.2),
    );
    target.clear(color::BLACK);
    target.render_iter(&camera, scene.iter());
    let pixel = center(&mut target);
    assert!(
        pixel[0].abs_diff(128) <= 1 && pixel[2].abs_diff(128) <= 1,
        "{pixel:?}"
    );
    let mut behind_transparent = Scene::default();
    behind_transparent.add(&quad.fill_color(color::GREEN).move_to(Vec3::Z * 0.5));
    target.render_iter(&camera, behind_transparent.iter());
    assert_eq!(center(&mut target), [0, 255, 0, 255]);
    let mut overlay = Scene::default();
    overlay.add(
        &quad
            .fill_color(color::RED)
            .depth(DepthMode::Overlay)
            .move_to(Vec3::Z * 0.9),
    );
    target.render_iter(&camera, overlay.iter());
    assert_eq!(center(&mut target), [255, 0, 0, 255]);
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn indexed_mesh_vertex_tints_use_the_existing_texture_and_scene_path() {
    use wgame::{
        glam::Vec4,
        shapes::{Mesh, shader::Vertex},
    };
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let mut target = Offscreen::new(&gfx, (32, 32));
    let mesh = Mesh::from_arrays(
        lib.shapes().state(),
        &[
            Vertex::new(Vec4::new(-1.0, -1.0, 0.2, 1.0), Vec3::new(0.0, 0.0, 1.0))
                .with_color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
            Vertex::new(Vec4::new(3.0, -1.0, 0.2, 1.0), Vec3::new(1.0, 0.0, 1.0))
                .with_color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
            Vertex::new(Vec4::new(-1.0, 3.0, 0.2, 1.0), Vec3::new(0.0, 1.0, 1.0))
                .with_color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        ],
        Some(&[0, 1, 2]),
    );
    let mut scene = Scene::default();
    scene.add(&lib.shapes().mesh(mesh).fill_color(color::WHITE));
    target.clear(color::BLACK);
    target.render(&Camera::new(&gfx, Mat4::IDENTITY), &scene.bake());
    assert_eq!(center(&mut target), [255, 0, 0, 255]);
}

#[cfg(feature = "3d")]
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn sphere_renders_through_the_shared_perspective_camera() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let mut target = Offscreen::new(&gfx, (64, 64));
    let camera = Camera::new(
        &gfx,
        wgame::glam::camera::rh::proj::directx::perspective(1.0, 1.0, 0.1, 10.0)
            * wgame::glam::camera::rh::view::look_at_mat4(
                Vec3::new(0.0, -3.0, 0.0),
                Vec3::ZERO,
                Vec3::Z,
            ),
    );
    let ray = camera.screen_ray(Vec2::splat(32.0), target.size()).unwrap();
    assert!((ray.1 - Vec3::Y).length() < 1e-5);
    let mut scene = Scene::default();
    scene.add(&lib.shapes().sphere(24, 12).fill_color(color::RED));
    target.clear(color::BLACK);
    target.render_iter(&camera, scene.iter());
    assert_eq!(center(&mut target), [255, 0, 0, 255]);
    assert_eq!(&support::pixels(&mut target)[..4], &[0, 0, 0, 255]);
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn offscreen_discard_preserves_initialization_and_submitted_depth() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let mut target = Offscreen::new(&gfx, (32, 32));
    let camera = Camera::new(&gfx, Mat4::IDENTITY);
    let mut near = Scene::default();
    near.add(
        &lib.shapes()
            .unit_quad()
            .fill_color(color::RED)
            .move_to(Vec3::Z * 0.2),
    );
    target.discard();
    target.render_iter(&camera, near.iter());
    assert_eq!(center(&mut target), [255, 0, 0, 255]);
    // Discarding an encoded clear must retain the depth already on the GPU.
    target.clear_depth();
    target.discard();
    let mut far = Scene::default();
    far.add(
        &lib.shapes()
            .unit_quad()
            .fill_color(color::BLUE)
            .move_to(Vec3::Z * 0.8),
    );
    target.render_iter(&camera, far.iter());
    assert_eq!(center(&mut target), [255, 0, 0, 255]);
}
