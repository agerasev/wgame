use super::*;
use wgame::{
    gfx::Graphics,
    image::{ImageRead, ImageWrite},
    texture::{RenderTexture, RenderTextureError, TextureSettings},
};

fn render_sample(
    target: &mut Offscreen,
    library: &Library,
    source: &dyn wgame::texture::SampledTexture,
) {
    let (width, height) = target.size();
    let camera = target.physical_camera();
    let mut scene = Scene::default();
    scene.add(
        &library
            .shapes()
            .rectangle((Vec2::ZERO, Vec2::new(width as f32, height as f32)))
            .fill_texture(source),
    );
    target.render(&camera, &scene.bake());
}
fn pixel(data: &[u8], width: usize, x: usize, y: usize) -> [u8; 4] {
    data[(y * width + x) * 4..][..4].try_into().unwrap()
}
fn near(actual: [u8; 4], expected: [u8; 4]) {
    assert!(
        actual.iter().zip(expected).all(|(a, e)| a.abs_diff(e) <= 2),
        "{actual:?} != {expected:?}"
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn render_texture_snapshot_and_sampling_retain_pixels_and_ownership() {
    let gfx = support::graphics();
    let library = Library::new(&gfx);
    let mut source = library
        .texturing()
        .render_texture((13, 7), TextureSettings::nearest())
        .unwrap();
    source.clear(color::RED);
    source
        .viewport((0, 3), (13, 4))
        .unwrap()
        .clear(color::GREEN);
    // Readback submits these draws and strips the 256-byte row padding.
    let mut snapshot = futures::executor::block_on(source.readback()).unwrap();
    assert_eq!(snapshot.data().len(), 13 * 7);
    assert_eq!(snapshot.data()[0], color::RED.to_rgba_f16());
    assert_eq!(snapshot.data()[3 * 13], color::GREEN.to_rgba_f16());
    // Cancelling after starting a transfer must release the mapping/borrow and
    // leave subsequent snapshots usable, even while device polling is finishing.
    {
        let mut cancelled = Box::pin(source.readback());
        let mut context = std::task::Context::from_waker(futures::task::noop_waker_ref());
        let _ = std::future::Future::poll(cancelled.as_mut(), &mut context);
    }
    let again = futures::executor::block_on(source.readback()).unwrap();
    assert_eq!(again.data(), snapshot.data());
    snapshot.data_mut().fill(color::BLUE.to_rgba_f16());
    let copied = library.make_texture(&snapshot, TextureSettings::nearest());
    let sample = source.sample();
    assert_eq!(sample.resource(), source.sample().resource());
    let mut target = Offscreen::new(&gfx, (13, 7));
    target.clear(color::BLACK);
    render_sample(&mut target, &library, &source);
    let expected = support::pixels(&mut target);
    assert_eq!(pixel(&expected, 13, 2, 0), [255, 0, 0, 255]);
    assert_eq!(pixel(&expected, 13, 2, 6), [0, 255, 0, 255]);
    source.clear(color::BLUE);
    source.discard();
    drop(source);
    target.clear(color::BLACK);
    render_sample(&mut target, &library, &sample);
    assert_eq!(support::pixels(&mut target), expected);
    target.clear(color::BLACK);
    render_sample(&mut target, &library, &copied);
    assert!(
        support::pixels(&mut target)
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [0, 0, 255, 255])
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn baked_sampling_observes_submitted_draws_and_replacement_keeps_old_allocation() {
    let gfx = support::graphics();
    let library = Library::new(&gfx);
    let mut source = library
        .texturing()
        .render_texture((9, 5), TextureSettings::linear())
        .unwrap();
    source.clear(color::RED);
    source.submit();
    let mut scene = Scene::default();
    scene.add(&library.shapes().unit_quad().fill_texture(&source));
    scene.add(&library.shapes().unit_quad().fill_texture(&source));
    assert_eq!(
        scene.len(),
        1,
        "one GPU allocation must batch across samples"
    );
    let baked = scene.bake();
    let mut target = Offscreen::new(&gfx, (16, 16));
    let camera = wgame::gfx::Camera::new(&gfx, wgame::glam::Mat4::IDENTITY);
    source.clear(color::GREEN);
    target.clear(color::BLACK);
    target.render(&camera, &baked);
    // Source drawing must execute before the dependent sampling, in one submission.
    gfx.queue().submit([source.finish(), target.finish()]);
    assert_eq!(
        pixel(&support::pixels(&mut target), 16, 8, 8),
        [0, 255, 0, 255]
    );
    source = library
        .texturing()
        .render_texture((17, 11), TextureSettings::linear())
        .unwrap();
    source.clear(color::BLUE);
    source.submit();
    target.clear(color::BLACK);
    target.render(&camera, &baked);
    assert_eq!(
        pixel(&support::pixels(&mut target), 16, 8, 8),
        [0, 255, 0, 255]
    );
    target.clear(color::BLACK);
    render_sample(&mut target, &library, &source);
    assert_eq!(
        pixel(&support::pixels(&mut target), 16, 8, 8),
        [0, 0, 255, 255]
    );
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn transparent_rendering_and_viewport_clears_composite_once() {
    let gfx = support::graphics();
    let library = Library::new(&gfx);
    let mut source = library
        .texturing()
        .render_texture((16, 16), TextureSettings::linear())
        .unwrap();
    let camera = source.physical_camera();
    let mut scene = Scene::default();
    scene.add(
        &library
            .shapes()
            .rectangle((Vec2::ZERO, Vec2::new(8.0, 16.0)))
            .fill_color(Rgba::new(1.0, 0.0, 0.0, 0.5)),
    );
    source.render(&camera, &scene.bake());
    source
        .viewport((8, 0), (8, 16))
        .unwrap()
        .clear(Rgba::new(0.0, 1.0, 0.0, 0.5));
    let snapshot = futures::executor::block_on(source.readback()).unwrap();
    assert!((snapshot.data()[0].r.to_f32() - 1.0).abs() < 0.01);
    assert!((snapshot.data()[0].a.to_f32() - 0.5).abs() < 0.01);
    let cpu = library.make_texture(&snapshot, TextureSettings::linear());
    let mut target = Offscreen::new(&gfx, (16, 16));
    for texture in [&source as &dyn wgame::texture::SampledTexture, &cpu] {
        target.clear(color::BLUE);
        render_sample(&mut target, &library, texture);
        let actual = support::pixels(&mut target);
        near(pixel(&actual, 16, 3, 5), [128, 0, 127, 255]);
        near(pixel(&actual, 16, 12, 5), [0, 128, 127, 255]);
    }
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn snapshots_support_native_formats_and_reject_invalid_creation() {
    let base = support::graphics();
    for format in [
        wgpu::TextureFormat::Bgra8Unorm,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureFormat::Rgba16Float,
    ] {
        let graphics = Graphics::new(
            base.adapter().clone(),
            base.device().clone(),
            base.queue().clone(),
            format,
        );
        let library = Library::new(&graphics);
        let textures = library.texturing();
        let mut source = textures
            .render_texture((3, 2), TextureSettings::linear())
            .unwrap();
        source.clear(Rgba::new(0.25, 0.5, 1.0, 0.5));
        let image = futures::executor::block_on(source.readback()).unwrap();
        let p = image.data()[0];
        let expected = [0.25, 0.5, 1.0, 0.5];
        for (a, e) in [p.r, p.g, p.b, p.a].into_iter().zip(expected) {
            assert!((a.to_f32() - e).abs() < 0.012, "{format:?}: {p:?}");
        }
        // A snapshot uploaded to the ordinary float atlas samples like its source,
        // including when storage uses sRGB encoding or BGRA channel ordering.
        let cpu = library.make_texture(&image, TextureSettings::linear());
        let mut copy = textures
            .render_texture((3, 2), TextureSettings::linear())
            .unwrap();
        copy.clear(color::BLACK);
        let camera = copy.physical_camera();
        let mut scene = Scene::default();
        scene.add(
            &library
                .shapes()
                .rectangle((Vec2::ZERO, Vec2::new(3.0, 2.0)))
                .fill_texture(&cpu),
        );
        copy.render(&camera, &scene.bake());
        let copied = futures::executor::block_on(copy.readback()).unwrap();
        for (a, e) in [copied.data()[0].r, copied.data()[0].g, copied.data()[0].b]
            .into_iter()
            .zip([0.125, 0.25, 0.5])
        {
            assert!((a.to_f32() - e).abs() < 0.012, "{format:?}: {a:?}");
        }
        assert!(matches!(
            textures.render_texture((0, 1), TextureSettings::linear()),
            Err(RenderTextureError::InvalidSize)
        ));
    }
    let graphics = Graphics::new(
        base.adapter().clone(),
        base.device().clone(),
        base.queue().clone(),
        wgpu::TextureFormat::R8Uint,
    );
    let textures = wgame::texture::TexturingLibrary::new(&graphics);
    assert!(matches!(
        RenderTexture::new(textures.state(), (2, 2), TextureSettings::nearest()),
        Err(RenderTextureError::UnsupportedFormat(
            wgpu::TextureFormat::R8Uint
        ))
    ));
}
