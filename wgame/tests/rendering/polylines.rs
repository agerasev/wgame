use super::*;
use wgame::{glam::Affine2, shapes::PolylinePoint, texture::TextureSettings};

fn gradient(lib: &Library) -> wgame::texture::Texture {
    lib.make_texture(
        &Image::with_data(
            (16, 16),
            (0..16)
                .flat_map(|y| {
                    (0..16).map(move |x| {
                        Rgba::new((x as f32 + 0.5) / 16.0, (y as f32 + 0.5) / 16.0, 0.25, 1.0)
                            .to_rgba_f16()
                    })
                })
                .collect::<Vec<_>>(),
        ),
        TextureSettings::nearest(),
    )
}

fn barycentric(p: Vec2, [a, b, c]: [Vec2; 3]) -> [f32; 3] {
    let area = (b - a).perp_dot(c - a);
    [
        (b - p).perp_dot(c - p) / area,
        (c - p).perp_dot(a - p) / area,
        (a - p).perp_dot(b - p) / area,
    ]
}

fn check_gradient(pixel: &[u8], uv: Vec2) {
    let expected = [
        ((uv.x * 16.0).floor() + 0.5) / 16.0 * 255.0,
        ((uv.y * 16.0).floor() + 0.5) / 16.0 * 255.0,
        0.25 * 255.0,
        255.0,
    ];
    for (&actual, expected) in pixel.iter().zip(expected) {
        assert!(
            (actual as f32 - expected).abs() < 2.0,
            "{pixel:?}, uv {uv:?}, expected {expected}"
        );
    }
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn four_point_quad_interpolates_uvs_in_the_destination_triangles() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let tex = gradient(&lib);
    let corners = [
        Vec2::new(8.0, 8.0),
        Vec2::new(58.0, 14.0),
        Vec2::new(42.0, 58.0),
        Vec2::new(12.0, 46.0),
    ];
    let uvs = [Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y];
    let [a, b, c, d] = corners;
    let mut scene = Scene::default();
    scene.add(&lib.shapes().quad(a, b, c, d).fill_texture(&tex));
    let mut target = Offscreen::new(&gfx, (64, 64));
    target.clear(color::BLACK);
    let camera = target.physical_camera();
    target.render_iter(&camera, scene.iter());
    let pixels = support::pixels(&mut target);
    let mut checked = 0;
    for y in 0..64 {
        for x in 0..64 {
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            for indices in [[0, 1, 2], [0, 2, 3]] {
                let weights = barycentric(p, indices.map(|i| corners[i]));
                if weights.iter().all(|w| *w > 0.02) {
                    let uv: Vec2 = indices
                        .into_iter()
                        .zip(weights)
                        .map(|(i, w)| uvs[i] * w)
                        .sum();
                    // Avoid nearest-filter boundaries, where rounding may select
                    // either of the adjacent texels.
                    let phase = (uv * 16.0).fract();
                    if phase.min_element() > 0.01 && phase.max_element() < 0.99 {
                        check_gradient(&pixels[(y * 64 + x) * 4..][..4], uv);
                        checked += 1;
                    }
                }
            }
        }
    }
    assert!(checked > 1000);
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn ribbon_uvs_precede_texture_transforms_and_follow_atlas_relocation() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let tex = gradient(&lib).transform_coord(Affine2::from_scale(Vec2::new(0.5, 1.0)));
    let ribbon = lib
        .shapes()
        .polyline(&[
            PolylinePoint {
                position: Vec2::new(8.0, 32.0),
                width: 12.0,
            },
            PolylinePoint {
                position: Vec2::new(24.0, 32.0),
                width: 20.0,
            },
            PolylinePoint {
                position: Vec2::new(56.0, 32.0),
                width: 0.0,
            },
        ])
        .fill_texture(&tex)
        .transform_texcoord(Affine2::from_translation(Vec2::new(0.25, 0.0)));
    let mut scene = Scene::default();
    scene.add(&ribbon);
    assert_eq!(scene.len(), 1, "segments share an instance batch");
    let mut target = Offscreen::new(&gfx, (64, 64));
    let camera = target.physical_camera();
    let render = |target: &mut Offscreen| {
        target.clear(color::BLACK);
        target.render_iter(&camera, scene.iter());
        support::pixels(target)
    };
    let first = render(&mut target);
    // Independently interpolate the two ribbon quads, including the zero-width
    // tip. These input lengths differ, but their U intervals are both 0.5.
    let vertices = [
        (Vec2::new(8.0, 38.0), Vec2::new(0.0, 0.0)),
        (Vec2::new(24.0, 42.0), Vec2::new(0.5, 0.0)),
        (Vec2::new(24.0, 22.0), Vec2::new(0.5, 1.0)),
        (Vec2::new(8.0, 26.0), Vec2::new(0.0, 1.0)),
        (Vec2::new(56.0, 32.0), Vec2::new(1.0, 0.0)),
        (Vec2::new(56.0, 32.0), Vec2::new(1.0, 1.0)),
    ];
    for (x, y) in [(10, 30), (18, 34), (23, 28), (24, 28), (32, 30), (45, 31)] {
        let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
        let uv = [[0, 1, 2], [0, 2, 3], [1, 5, 2]]
            .into_iter()
            .find_map(|indices| {
                let weights = barycentric(p, indices.map(|i| vertices[i].0));
                weights.iter().all(|w| *w >= 0.0).then(|| {
                    indices
                        .into_iter()
                        .zip(weights)
                        .map(|(i, w)| vertices[i].1 * w)
                        .sum::<Vec2>()
                })
            })
            .expect("sample lies inside the ribbon");
        check_gradient(
            &first[(y * 64 + x) * 4..][..4],
            Vec2::new(0.25 + 0.5 * uv.x, uv.y),
        );
    }
    let baked = scene.bake();
    let _large = lib.make_texture(
        &Image::with_color((512, 512), color::BLUE.to_rgba_f16()),
        Default::default(),
    );
    assert_eq!(first, render(&mut target));
    target.clear(color::BLACK);
    target.render(&camera, &baked);
    assert_eq!(first, support::pixels(&mut target));
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn translucent_miters_and_bevels_do_not_double_blend_their_shared_edges() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    for last in [Vec2::new(56.0, 8.0), Vec2::new(8.0, 28.0)] {
        let mut scene = Scene::default();
        scene.add(
            &lib.shapes()
                .polyline(&[
                    PolylinePoint {
                        position: Vec2::new(8.0, 48.0),
                        width: 6.0,
                    },
                    PolylinePoint {
                        position: Vec2::new(56.0, 48.0),
                        width: 6.0,
                    },
                    PolylinePoint {
                        position: last,
                        width: 6.0,
                    },
                ])
                .fill_color(Rgba::new(1.0, 0.0, 0.0, 0.5)),
        );
        assert_eq!(scene.len(), 1, "bevels share the quad batch");
        let mut target = Offscreen::new(&gfx, (64, 64));
        target.clear(color::BLACK);
        let camera = target.physical_camera();
        target.render_iter(&camera, scene.iter());
        let pixels = support::pixels(&mut target);
        let mut filled = 0;
        for pixel in pixels.as_chunks::<4>().0 {
            if pixel[0] > 0 {
                assert!(
                    pixel[0].abs_diff(128) <= 1,
                    "overlap at {pixel:?}, endpoint {last:?}"
                );
                filled += 1;
            }
        }
        assert!(filled > 400);
        for (x, y) in [(16, 48), (40, 48), (55, 48)] {
            assert!(
                pixels[(y * 64 + x) * 4].abs_diff(128) <= 1,
                "hole at {x},{y}"
            );
        }
    }
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn empty_polylines_add_no_batch_and_render_nothing() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let point = PolylinePoint {
        position: Vec2::splat(16.0),
        width: 10.0,
    };
    let mut scene = Scene::default();
    for points in [vec![], vec![point], vec![point, point]] {
        scene.add(&lib.shapes().polyline(&points).fill_color(color::WHITE));
    }
    assert!(scene.is_empty());
    let mut target = Offscreen::new(&gfx, (32, 32));
    let camera = target.physical_camera();
    target.clear(color::BLACK);
    target.render_iter(&camera, scene.iter());
    assert!(
        support::pixels(&mut target)
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| *pixel == [0, 0, 0, 255])
    );
}
