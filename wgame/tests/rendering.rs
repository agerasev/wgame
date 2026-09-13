//! Offscreen pixel reference checks; run explicitly on a GPU or Mesa lavapipe.
//!
//! ```sh
//! cargo test --locked -p wgame --test rendering -- --ignored
//! ```
//!
//! These tests are ignored by the ordinary suite so CPU-only machines can run it.
//! Explicit execution requires an adapter and fails if none is available. The
//! helper renders into an offscreen texture and reads pixels back without a
//! window; Xvfb is only needed for the separate playground smoke check.
//! On headless Linux, install Mesa Vulkan and set `WGPU_BACKEND=vulkan`.
//! Pixel checks use analytic expectations or comparisons, not a complete visual
//! gallery or a guarantee of typography completeness and driver portability.
#![cfg(all(feature = "shapes", feature = "image", feature = "typography"))]
mod support;
use wgame::gfx::types::Color;
use wgame::{
    Library,
    gfx::types::color,
    gfx::{Offscreen, Scene},
    glam::Vec2,
    image::Image,
    prelude::*,
    rgb::Rgba,
};

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn transparency_preserves_insertion_order() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let mut target = Offscreen::new(&gfx, (64, 64));
    let camera = target.physical_camera();
    let red = lib
        .shapes()
        .rectangle((Vec2::ZERO, Vec2::splat(64.0)))
        .fill_color(Rgba::new(1.0, 0.0, 0.0, 0.5));
    let blue = lib
        .shapes()
        .unit_circle()
        .fill_color(Rgba::new(0.0, 0.0, 1.0, 0.5))
        .scale(24.0)
        .move_to(Vec2::splat(32.0));
    let mut scene = Scene::default();
    scene.add(&red);
    scene.add(&blue);
    scene.add(&red);
    assert_eq!(scene.len(), 3);
    target.clear(color::BLACK);
    target.render_iter(&camera, scene.iter());
    let data = support::pixels(&mut target);
    let baked = scene.bake();
    target.clear(color::BLACK);
    target.render(&camera, &baked);
    assert_eq!(data, support::pixels(&mut target));
    let center = &data[(32 * 64 + 32) * 4..(32 * 64 + 32) * 4 + 4];
    for (actual, expected) in center.iter().zip([159u8, 0, 64, 255]) {
        assert!(actual.abs_diff(expected) <= 2, "pixel {center:?}");
    }
}
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn atlas_growth_and_updates_render_correctly() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let image = Image::with_color((2, 2), color::RED.to_rgba_f16());
    let tex = lib.make_texture(&image, wgame::texture::TextureSettings::linear());
    let mut target = Offscreen::new(&gfx, (64, 64));
    let camera = target.physical_camera();
    let shape = lib
        .shapes()
        .rectangle((Vec2::ZERO, Vec2::splat(64.0)))
        .fill_texture(&tex);
    let mut scene = Scene::default();
    scene.add(&shape);
    let baked = scene.bake();
    let render = |target: &mut Offscreen| {
        target.clear(color::BLACK);
        target.render_iter(&camera, scene.iter());
        support::pixels(target)
    };
    let before = render(&mut target);
    assert_eq!(
        &before[(32 * 64 + 32) * 4..(32 * 64 + 32) * 4 + 4],
        &[255, 0, 0, 255]
    );
    let _large = lib.make_texture(
        &Image::with_color((512, 512), color::BLUE.to_rgba_f16()),
        Default::default(),
    );
    let after = render(&mut target);
    assert_eq!(before, after);
    target.clear(color::BLACK);
    target.render(&camera, &baked);
    assert_eq!(before, support::pixels(&mut target));
    tex.update(|mut dst| {
        use wgame::image::ImageWriteMut;
        for (_, p) in dst.pixels_mut() {
            *p = color::GREEN.to_rgba_f16();
        }
    });
    let updated = render(&mut target);
    assert_eq!(
        &updated[(32 * 64 + 32) * 4..(32 * 64 + 32) * 4 + 4],
        &[0, 255, 0, 255]
    );
}
#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn text_survives_glyph_growth_and_target_resize() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let font = lib.make_font(
        &wgame::typography::FontData::new(
            include_bytes!("../../wgame-examples/assets/free-sans-bold.ttf").to_vec(),
            0,
        )
        .unwrap(),
    );
    let raster = font.rasterize(20.0);
    let text = raster.text("Ag").scale(20.0).move_to(Vec2::new(4.0, 26.0));
    let render = |size| {
        let mut target = Offscreen::new(&gfx, size);
        let camera = target.physical_camera();
        target.clear(color::BLACK);
        let mut scene = Scene::default();
        scene.add(&text);
        target.render_iter(&camera, scene.iter());
        support::pixels(&mut target)
    };
    let first = render((64, 64));
    assert!(first.chunks(4).any(|p| p[0] > 100));
    let chars: String = (32..2000).filter_map(char::from_u32).collect();
    let _other = raster.text(&chars);
    assert_eq!(first, render((64, 64)));
    let larger = render((128, 128));
    for y in 0..64 {
        assert_eq!(
            &first[y * 256..(y + 1) * 256],
            &larger[y * 512..y * 512 + 256]
        );
    }
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn partial_texture_updates_preserve_neighbors_and_filtering_border() {
    use wgame::image::{ImageBase, ImageReadExt, ImageWriteMut};
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let tex = lib.make_texture(
        &Image::with_color((4, 4), color::RED.to_rgba_f16()),
        Default::default(),
    );
    for (x, y) in [(2, 2), (0, 2), (3, 1), (2, 0), (1, 3), (3, 3)] {
        tex.update_part(
            |mut dst| *dst.get_mut((0, 0).into()) = color::BLUE.to_rgba_f16(),
            wgame::image::euclid::rect(x, y, 1, 1),
        );
        tex.with(|src| assert_eq!(*src.get((x, y).into()), color::BLUE.to_rgba_f16()));
    }
    tex.update_part(
        |dst| assert_eq!(dst.size().width, 0),
        wgame::image::euclid::rect(0, 0, 0, 0),
    );
    tex.image().with(|img| {
        assert_eq!(img.size().width, 6);
        for i in 1..5 {
            assert_eq!(img.get((0, i).into()), img.get((1, i).into()));
            assert_eq!(img.get((5, i).into()), img.get((4, i).into()));
            assert_eq!(img.get((i, 0).into()), img.get((i, 1).into()));
            assert_eq!(img.get((i, 5).into()), img.get((i, 4).into()));
        }
        assert_eq!(img.get((5, 5).into()), img.get((4, 4).into()));
    });
    tex.resize((6, 6));
    tex.with(|src| {
        assert_eq!(*src.get((3, 3).into()), color::BLUE.to_rgba_f16());
        assert_eq!(
            *src.get((4, 3).into()),
            Default::default(),
            "old border must not become content"
        );
    });
    tex.resize((2, 2));
    tex.image()
        .with(|img| assert_eq!(img.get((3, 3).into()), img.get((2, 2).into())));
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn baked_and_encoded_draws_survive_dropped_atlas_items() {
    use wgame::{
        image::{Atlas, ImageWriteMut},
        texture::TextureAtlas,
    };
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    // Append within one generation, compact at the same size, and grow.
    for (old_size, new_size, generation, atlas_size) in [
        (6, 6, 0, (16, 16)),
        (14, 6, 1, (16, 16)),
        (14, 14, 1, (32, 16)),
    ] {
        for encoded in [false, true] {
            let atlas = TextureAtlas::new(
                lib.texturing().state(),
                Atlas::with_size((16, 16).into()),
                wgpu::TextureFormat::Rgba16Float,
            );
            let tex = atlas.allocate((old_size, old_size), Default::default());
            tex.update(|mut dst| {
                for (_, p) in dst.pixels_mut() {
                    *p = color::RED.to_rgba_f16();
                }
            });
            let shape = lib
                .shapes()
                .rectangle((Vec2::ZERO, Vec2::splat(16.0)))
                .fill_texture(&tex);
            let mut scene = Scene::default();
            scene.add(&shape);
            let baked = scene.bake();
            let mut target = Offscreen::new(&gfx, (16, 16));
            let camera = target.physical_camera();
            target.clear(color::BLACK);
            let baked = if encoded {
                target.render(&camera, &baked);
                drop(baked);
                None
            } else {
                Some(baked)
            };
            drop(scene);
            drop(shape);
            drop(tex);
            let replacement = atlas.allocate((new_size, new_size), Default::default());
            assert_eq!(atlas.inner().generation(), generation);
            assert_eq!(atlas.inner().size(), atlas_size.into());
            replacement.update(|mut dst| {
                for (_, p) in dst.pixels_mut() {
                    *p = color::BLUE.to_rgba_f16();
                }
            });
            let mut replacement_scene = Scene::default();
            replacement_scene.add(
                &lib.shapes()
                    .rectangle((Vec2::ZERO, Vec2::splat(16.0)))
                    .fill_texture(&replacement),
            );
            let mut other_target = Offscreen::new(&gfx, (16, 16));
            other_target.clear(color::BLACK);
            other_target.render_iter(&camera, replacement_scene.iter());
            let blue = support::pixels(&mut other_target);
            assert!(
                blue.as_chunks::<4>()
                    .0
                    .iter()
                    .all(|p| *p == [0, 0, 255, 255])
            );
            if let Some(baked) = baked {
                target.render(&camera, &baked);
            }
            let red = support::pixels(&mut target);
            assert!(
                red.as_chunks::<4>()
                    .0
                    .iter()
                    .all(|p| *p == [255, 0, 0, 255]),
                "old size {old_size}, new size {new_size}, encoded {encoded}"
            );
        }
    }
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn resizing_keeps_baked_pixels_and_rebaking_resolves_new_coordinates() {
    use wgame::{
        image::{Atlas, ImageWriteMut},
        texture::TextureAtlas,
    };
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let atlas = TextureAtlas::new(
        lib.texturing().state(),
        Atlas::with_size((32, 32).into()),
        wgpu::TextureFormat::Rgba16Float,
    );
    let tex = atlas.allocate((4, 4), Default::default());
    tex.update(|mut dst| {
        for (_, p) in dst.pixels_mut() {
            *p = color::RED.to_rgba_f16();
        }
    });
    let mut scene = Scene::default();
    scene.add(
        &lib.shapes()
            .rectangle((Vec2::ZERO, Vec2::splat(16.0)))
            .fill_texture(&tex),
    );
    let baked = scene.bake();
    let old_rect = tex.image().rect();
    tex.resize((6, 6));
    assert_eq!(atlas.inner().generation(), 0);
    assert!(!old_rect.intersects(&tex.image().rect()));
    tex.update(|mut dst| {
        for (_, p) in dst.pixels_mut() {
            *p = color::BLUE.to_rgba_f16();
        }
    });
    let fresh = scene.bake();
    let mut target = Offscreen::new(&gfx, (16, 16));
    let camera = target.physical_camera();
    for (renderer, expected) in [(&baked, [255, 0, 0, 255]), (&fresh, [0, 0, 255, 255])] {
        target.clear(color::BLACK);
        target.render(&camera, renderer);
        assert!(
            support::pixels(&mut target)
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| *p == expected)
        );
    }
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn baked_text_survives_glyph_repacking_without_outer_atlas_growth() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let font = lib.make_font(
        &wgame::typography::FontData::new(
            include_bytes!("../../wgame-examples/assets/free-sans-bold.ttf").to_vec(),
            0,
        )
        .unwrap(),
    );
    let raster = font.rasterize(20.0);
    let outer = raster.inner().atlas().inner();
    // Leave room for several font-local reallocations in the same GPU texture.
    drop(outer.allocate((1024, 1024)));
    let text = raster.text("Ag").scale(20.0).move_to(Vec2::new(4.0, 26.0));
    let mut scene = Scene::default();
    scene.add(&text);
    let baked = scene.bake();
    let mut target = Offscreen::new(&gfx, (64, 64));
    let camera = target.physical_camera();
    target.clear(color::BLACK);
    target.render(&camera, &baked);
    let before = support::pixels(&mut target);
    assert!(before.as_chunks::<4>().0.iter().any(|p| p[0] > 100));
    let generation = outer.generation();
    let old_rect = raster.image().rect();
    raster.add_chars(32u32..2000);
    assert_eq!(outer.generation(), generation);
    assert_ne!(raster.image().rect().size, old_rect.size);
    assert!(!raster.image().rect().intersects(&old_rect));
    // Reuse the original scene: glyph coordinates are resolved at bake time.
    let fresh = scene.bake();
    drop(scene);
    drop(text);
    drop(raster);
    drop(font);
    for renderer in [&fresh, &baked] {
        target.clear(color::BLACK);
        target.render(&camera, renderer);
        assert_eq!(before, support::pixels(&mut target));
    }
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn populated_cpu_atlas_is_uploaded_on_first_gpu_use() {
    use wgame::{
        image::AtlasImage,
        texture::{Texture, TextureAtlas},
    };
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let image = AtlasImage::from_single(Image::with_color((16, 16), color::RED.to_rgba_f16()));
    let atlas = TextureAtlas::new(
        lib.texturing().state(),
        image.atlas(),
        wgpu::TextureFormat::Rgba16Float,
    );
    let tex = Texture::new(&atlas, image, Default::default());
    let mut scene = Scene::default();
    scene.add(
        &lib.shapes()
            .rectangle((Vec2::ZERO, Vec2::splat(16.0)))
            .fill_texture(&tex),
    );
    let mut target = Offscreen::new(&gfx, (16, 16));
    let camera = target.physical_camera();
    target.clear(color::BLACK);
    target.render_iter(&camera, scene.iter());
    assert!(
        support::pixels(&mut target)
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [255, 0, 0, 255])
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn lines_preserve_width_endpoints_and_camera_picking() {
    let gfx = support::graphics();
    let lib = Library::new(&gfx);
    let mut target = Offscreen::new(&gfx, (64, 64));
    let camera = target.physical_camera();
    assert!(
        (camera
            .screen_to_world(Vec2::new(12.0, 20.0), (64, 64))
            .unwrap()
            - Vec2::new(12.0, 20.0))
        .length()
            < 0.001
    );
    let mut scene = Scene::default();
    scene.add(
        &lib.shapes()
            .line(Vec2::new(8.0, 12.0), Vec2::new(32.0, 12.0), 8.0)
            .fill_color(color::RED),
    );
    scene.add(
        &lib.shapes()
            .line(Vec2::new(48.0, 8.0), Vec2::new(48.0, 32.0), 8.0)
            .fill_color(color::GREEN),
    );
    scene.add(
        &lib.shapes()
            .line(Vec2::new(8.0, 40.0), Vec2::new(24.0, 56.0), 6.0)
            .fill_color(color::BLUE),
    );
    scene.add(
        &lib.shapes()
            .line(Vec2::splat(60.0), Vec2::splat(60.0), 12.0)
            .fill_color(color::WHITE),
    );
    target.clear(color::BLACK);
    target.render_iter(&camera, scene.iter());
    let data = support::pixels(&mut target);
    let pixel = |x: usize, y: usize| &data[(y * 64 + x) * 4..(y * 64 + x) * 4 + 4];
    for (x, y, expected) in [
        (8, 8, [255, 0, 0, 255]),
        (31, 15, [255, 0, 0, 255]),
        (7, 12, [0, 0, 0, 255]),
        (32, 12, [0, 0, 0, 255]),
        (20, 16, [0, 0, 0, 255]),
        (44, 8, [0, 255, 0, 255]),
        (51, 31, [0, 255, 0, 255]),
        (52, 20, [0, 0, 0, 255]),
        (16, 48, [0, 0, 255, 255]),
        (20, 44, [0, 0, 0, 255]),
        (60, 60, [0, 0, 0, 255]),
    ] {
        assert_eq!(pixel(x, y), expected, "at {x},{y}");
    }
}
