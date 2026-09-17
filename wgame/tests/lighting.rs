//! Explicit GPU regression tests for the optional lighting materials.
#![cfg(all(feature = "3d", feature = "image"))]
mod support;
use wgame::{
    Library,
    gfx::{
        Camera, Offscreen, Scene,
        types::{Color, color},
    },
    glam::{Affine2, Mat4, Vec2, Vec3, Vec4},
    image::Image,
    prelude::*,
    shapes::{Mesh, PolygonFill, shader::Vertex},
    shapes3d::{LightParameters, Lighting, MaterialSettings, NormalY},
};

fn center(target: &mut Offscreen) -> [u8; 4] {
    let data = support::pixels(target);
    let i = (target.size().1 / 2 * target.size().0 + target.size().0 / 2) as usize * 4;
    data[i..i + 4].try_into().unwrap()
}
fn quad(lib: &Library, normal: Vec3, degenerate_uv: bool) -> PolygonFill {
    let vertices: Vec<_> = [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
        .into_iter()
        .map(|(x, y)| {
            let uv = if degenerate_uv {
                Vec2::ZERO
            } else {
                Vec2::new(x + 1.0, y + 1.0) * 0.5
            };
            Vertex::new(Vec4::new(x, y, 0.25, 1.0), uv.extend(1.0)).with_normal(normal)
        })
        .collect();
    lib.shapes()
        .mesh(Mesh::from_arrays(
            lib.shapes().state(),
            &vertices,
            Some(&[0, 1, 2, 2, 1, 3]),
        ))
        .fill_color(color::WHITE)
}
fn matte() -> MaterialSettings {
    MaterialSettings {
        specular: 0.0,
        albedo_srgb: false,
        ..Default::default()
    }
}
fn sunlight(direction: Vec3) -> LightParameters {
    LightParameters {
        ambient: Vec3::ZERO,
        direction,
        color: Vec3::ONE,
        eye: Vec3::Z * 4.0,
    }
}
fn encoded(linear: f32) -> u8 {
    let v = if linear <= 0.0031308 {
        12.92 * linear
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    };
    (v * 255.0).round() as u8
}
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn diffuse_normals_follow_inverse_transpose_and_reflections() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let mut target = Offscreen::new(&gfx, (32, 32));
    let camera = Camera::new(&gfx, Mat4::IDENTITY);
    let lighting = Lighting::new(lib.shapes(), lib.texturing(), sunlight(Vec3::Z)).unwrap();
    let material = lighting.material(None, matte()).unwrap();
    let n = (Vec3::X + Vec3::Z).normalize();
    let shape = quad(&lib, n, false).with_material(&material);
    for transform in [
        Mat4::IDENTITY,
        Mat4::from_scale(Vec3::new(2.0, 1.0, 0.5)),
        Mat4::from_scale(Vec3::new(-2.0, 1.0, 0.5)),
        Mat4::from_rotation_z(1.0),
    ] {
        for direction in [Vec3::Z, Vec3::X, -Vec3::X] {
            lighting.update(sunlight(direction)).unwrap();
            let mut scene = Scene::default();
            scene.add(&shape.transform(wgame::glam::Affine3A::from_mat4(transform)));
            target.clear(color::BLACK);
            target.render_iter(&camera, scene.iter());
            let actual = center(&mut target);
            let expected = encoded(
                transform
                    .inverse()
                    .transpose()
                    .transform_vector3(n)
                    .normalize()
                    .dot(direction)
                    .max(0.0),
            );
            assert!(
                actual[0].abs_diff(expected) <= 2,
                "{actual:?}, expected {expected} for {transform:?}/{direction:?}"
            );
        }
    }
}
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn normal_maps_respect_mirrored_uvs_green_conventions_and_degenerate_uvs() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let mut target = Offscreen::new(&gfx, (32, 32));
    let camera = Camera::new(&gfx, Mat4::IDENTITY);
    let lighting = Lighting::new(lib.shapes(), lib.texturing(), sunlight(Vec3::X)).unwrap();
    let tilt_x = lib.make_texture(
        &Image::with_color((1, 1), Vec4::new(1.0, 0.5, 0.5, 1.0).to_rgba_f16()),
        Default::default(),
    );
    let tilt_y = lib.make_texture(
        &Image::with_color((1, 1), Vec4::new(0.5, 1.0, 0.5, 1.0).to_rgba_f16()),
        Default::default(),
    );
    for (normal, direction, settings, degenerate, expected) in [
        (tilt_x.clone(), Vec3::X, matte(), false, 255),
        (
            tilt_x.transform_coord(
                Affine2::from_translation(Vec2::X) * Affine2::from_scale(Vec2::new(-1.0, 1.0)),
            ),
            Vec3::X,
            matte(),
            false,
            0,
        ),
        (tilt_y.clone(), Vec3::Y, matte(), false, 255),
        (
            tilt_y,
            Vec3::Y,
            MaterialSettings {
                normal_y: NormalY::Negative,
                ..matte()
            },
            false,
            0,
        ),
        (tilt_x, Vec3::Z, matte(), true, 255),
    ] {
        lighting.update(sunlight(direction)).unwrap();
        let material = lighting.material(Some(&normal), settings).unwrap();
        let mut scene = Scene::default();
        scene.add(&quad(&lib, Vec3::Z, degenerate).with_material(&material));
        target.clear(color::BLACK);
        target.render(&camera, &scene.bake());
        let actual = center(&mut target);
        assert!(
            actual[0].abs_diff(expected) <= 2,
            "{actual:?} != {expected}"
        );
    }
}
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn color_space_neutral_maps_atlas_relocation_and_live_lights() {
    use wgame::texture::TexturingLibrary;
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let normals = TexturingLibrary::new(&gfx);
    let mut target = Offscreen::new(&gfx, (32, 32));
    let camera = Camera::new(&gfx, Mat4::IDENTITY);
    let lighting = Lighting::new(lib.shapes(), lib.texturing(), sunlight(Vec3::Z)).unwrap();
    let base = lib.make_texture(
        &Image::with_color((2, 2), Vec4::new(0.5, 0.25, 0.125, 1.0).to_rgba_f16()),
        Default::default(),
    );
    let neutral = normals.texture(
        &Image::with_color((2, 2), Vec4::new(0.5, 0.5, 1.0, 1.0).to_rgba_f16()),
        Default::default(),
    );
    let settings = MaterialSettings {
        specular: 0.0,
        ..Default::default()
    };
    let material = lighting
        .material(Some(&neutral.multiply_color(color::RED)), settings)
        .unwrap();
    let mut scene = Scene::default();
    scene.add(
        &lib.shapes()
            .unit_quad()
            .fill_texture(&base)
            .with_material(&material),
    );
    let baked = scene.bake();
    target.clear(color::BLACK);
    target.render(&camera, &baked);
    let before = support::pixels(&mut target);
    let actual = &before[(16 * 32 + 16) * 4..][..4];
    assert!(
        actual[0].abs_diff(128) <= 1 && actual[1].abs_diff(64) <= 1 && actual[2].abs_diff(32) <= 1,
        "{actual:?}"
    );
    let _grow_color = lib.make_texture(
        &Image::with_color((512, 512), color::GREEN.to_rgba_f16()),
        Default::default(),
    );
    let _grow_normal = normals.texture(
        &Image::with_color((512, 512), color::RED.to_rgba_f16()),
        Default::default(),
    );
    for renderer in [&baked, &scene.bake()] {
        target.clear(color::BLACK);
        target.render(&camera, renderer);
        assert!(
            support::pixels(&mut target) == before,
            "atlas relocation changed material"
        );
    }
    let fallback = lighting.material(None, settings).unwrap();
    let mut fallback_scene = Scene::default();
    fallback_scene.add(
        &lib.shapes()
            .unit_quad()
            .fill_texture(&base)
            .with_material(&fallback),
    );
    target.clear(color::BLACK);
    target.render(&camera, &fallback_scene.bake());
    assert!(
        support::pixels(&mut target) == before,
        "neutral map differs from absent map"
    );
    lighting.update(sunlight(-Vec3::Z)).unwrap();
    target.clear(color::BLACK);
    target.render(&camera, &baked);
    assert_eq!(center(&mut target), [0, 0, 0, 255]);
}
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn specular_highlights_follow_the_eye_in_baked_scenes() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let mut target = Offscreen::new(&gfx, (32, 32));
    let camera = Camera::new(&gfx, Mat4::IDENTITY);
    let lighting = Lighting::new(lib.shapes(), lib.texturing(), sunlight(Vec3::Z)).unwrap();
    let material = lighting
        .material(
            None,
            MaterialSettings {
                specular: 0.5,
                shininess: 64.0,
                ..matte()
            },
        )
        .unwrap();
    let mut scene = Scene::default();
    scene.add(
        &lib.shapes()
            .unit_quad()
            .fill_color(color::BLACK)
            .with_material(&material),
    );
    let baked = scene.bake();
    target.clear(color::BLACK);
    target.render(&camera, &baked);
    let bright = center(&mut target)[0];
    lighting
        .update(LightParameters {
            eye: Vec3::new(10.0, 0.0, 0.1),
            ..sunlight(Vec3::Z)
        })
        .unwrap();
    target.clear(color::BLACK);
    target.render(&camera, &baked);
    let dark = center(&mut target)[0];
    assert!(bright > 180 && dark < 15, "bright {bright}, dark {dark}");
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn shared_triangle_primitive_and_srgb_targets_support_lighting() {
    use wgame::gfx::Graphics;
    let original = support::graphics();
    for format in [
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Rgba8UnormSrgb,
    ] {
        let gfx = Graphics::new(
            original.adapter().clone(),
            original.device().clone(),
            original.queue().clone(),
            format,
        );
        let lib = Library::new(&gfx);
        let lighting = Lighting::new(lib.shapes(), lib.texturing(), sunlight(Vec3::Y)).unwrap();
        let material = lighting
            .material(
                None,
                MaterialSettings {
                    specular: 0.0,
                    ..Default::default()
                },
            )
            .unwrap();
        // A triangle in XZ, with a zero first vertex: the reference geometry's
        // barycentric normal must survive a singular planar transform.
        let triangle = lib
            .shapes()
            .triangle(
                Vec3::ZERO,
                Vec3::new(0.0, 0.0, 3.0),
                Vec3::new(3.0, 0.0, 0.0),
            )
            .fill_color(Vec4::new(0.5, 0.5, 0.5, 1.0))
            .with_material(&material);
        let view =
            wgame::glam::camera::rh::proj::directx::orthographic(-1.0, 1.0, -1.0, 1.0, 0.1, 10.0)
                * wgame::glam::camera::rh::view::look_at_mat4(
                    Vec3::new(0.5, 2.0, 0.5),
                    Vec3::new(0.5, 0.0, 0.5),
                    Vec3::Z,
                );
        let camera = Camera::new(&gfx, view);
        let mut target = Offscreen::new(&gfx, (32, 32));
        let mut scene = Scene::default();
        scene.add(&triangle);
        target.clear(color::BLACK);
        target.render(&camera, &scene.bake());
        let actual = center(&mut target);
        // Instance tint is a linear multiplier on the white color texture.
        assert!(
            actual[0].abs_diff(encoded(0.5)) <= 2,
            "{format:?}: {actual:?}"
        );
    }
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn material_parameters_batch_and_unlit_objects_share_the_scene() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let lighting = Lighting::new(lib.shapes(), lib.texturing(), sunlight(Vec3::Z)).unwrap();
    let normal = lib.make_texture(
        &Image::with_color((1, 1), Vec4::new(1.0, 0.5, 0.5, 1.0).to_rgba_f16()),
        Default::default(),
    );
    let flat = lighting
        .material(
            Some(&normal),
            MaterialSettings {
                normal_strength: 0.0,
                ..matte()
            },
        )
        .unwrap();
    let bumpy = lighting.material(Some(&normal), matte()).unwrap();
    let base = lib
        .shapes()
        .unit_quad()
        .fill_color(color::WHITE)
        .transform(Affine2::from_scale(Vec2::new(0.5, 1.0)));
    let mut scene = Scene::default();
    scene.add(&base.with_material(&flat).move_to(Vec2::new(-0.5, 0.0)));
    scene.add(&base.with_material(&bumpy).move_to(Vec2::new(0.5, 0.0)));
    assert_eq!(
        scene.len(),
        1,
        "material parameters should be instanced, not separate pipelines"
    );
    let camera = Camera::new(&gfx, Mat4::IDENTITY);
    let mut target = Offscreen::new(&gfx, (32, 32));
    target.clear(color::BLACK);
    target.render(&camera, &scene.bake());
    let data = support::pixels(&mut target);
    assert_eq!(&data[(16 * 32 + 8) * 4..][..4], &[255, 255, 255, 255]);
    assert_eq!(&data[(16 * 32 + 24) * 4..][..4], &[0, 0, 0, 255]);
    scene.add(&lib.shapes().unit_quad().fill_color(color::BLUE).scale(0.2));
    target.clear(color::BLACK);
    target.render(&camera, &scene.bake());
    assert_eq!(center(&mut target), [0, 0, 255, 255]);
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn rendered_normal_maps_remain_live_in_baked_materials() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let lighting = Lighting::new(lib.shapes(), lib.texturing(), sunlight(Vec3::X)).unwrap();
    let mut normal = lib
        .texturing()
        .render_texture((4, 4), Default::default())
        .unwrap();
    // Partial alpha exercises unpremultiplication of normal data before decoding.
    normal.clear(Vec4::new(1.0, 0.5, 0.5, 0.5));
    normal.submit();
    let material = lighting.material(Some(&normal), matte()).unwrap();
    let mut scene = Scene::default();
    scene.add(&quad(&lib, Vec3::Z, false).with_material(&material));
    let baked = scene.bake();
    let mut target = Offscreen::new(&gfx, (16, 16));
    let camera = Camera::new(&gfx, Mat4::IDENTITY);
    target.clear(color::BLACK);
    target.render(&camera, &baked);
    assert!(center(&mut target)[0] >= 253);
    normal.clear(Vec4::new(0.0, 0.5, 0.5, 1.0));
    normal.submit();
    drop(normal);
    target.clear(color::BLACK);
    target.render(&camera, &baked);
    assert_eq!(center(&mut target), [0, 0, 0, 255]);
}
