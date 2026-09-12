//! Offscreen pixel reference checks; run explicitly on a GPU or Mesa lavapipe.
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
    let render = |target: &mut Offscreen| {
        target.clear(color::BLACK);
        let mut scene = Scene::default();
        scene.add(&shape);
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
