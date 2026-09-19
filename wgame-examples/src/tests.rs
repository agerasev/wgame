use super::Labels;
use wgame::{
    Library,
    gfx::{Graphics, Scene},
    glam::{Vec2, Vec4},
    image::ImageRead,
    prelude::*,
    typography::Text,
};

#[test]
#[ignore = "requires a GPU adapter; run with --ignored"]
fn labels_match_native_size_rasters_after_size_and_scale_changes() {
    futures::executor::block_on(async {
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
        let adapter = instance
            .request_adapter(&Default::default())
            .await
            .expect("GPU adapter required (Mesa lavapipe works)");
        let (device, queue) = adapter.request_device(&Default::default()).await.unwrap();
        let graphics = Graphics::new(adapter, device, queue, wgpu::TextureFormat::Rgba8Unorm);
        let library = Library::new(&graphics);
        let mut labels = Labels::new(&library).unwrap();
        let mut target = library
            .texturing()
            .render_texture((256, 128), Default::default())
            .unwrap();
        let camera = target.physical_camera();
        let mut retained = None;
        for scale in [1.0, 2.0, 0.75, 1.5, 1.0] {
            labels.set_scale(scale);
            for size in [12.0, 20.0, 32.0] {
                let actual = labels.text("Ag", size).scale(scale);
                let expected = labels
                    .font
                    .rasterize(size * scale)
                    .text("Ag")
                    .scale(size * scale);
                assert_eq!(actual.metrics().size(), size * scale);
                let mut results = Vec::new();
                for text in [&actual, &expected] {
                    target.clear(Vec4::ZERO);
                    let mut scene = Scene::default();
                    scene.add(&text.move_to(Vec2::new(8.0, 80.0)));
                    target.render(&camera, &scene.bake());
                    let pixels = target.readback().await.unwrap();
                    assert!(pixels.data().iter().any(|p| p.a.to_f32() > 0.5));
                    results.push(pixels.data().to_vec());
                }
                assert_eq!(results[0], results[1], "size {size}, scale {scale}");
                if retained.is_none() {
                    retained = Some((actual, results.remove(0)));
                }
            }
        }
        // Releasing old cache entries must not invalidate text held by a scene.
        let (text, pixels): (Text, _) = retained.unwrap();
        target.clear(Vec4::ZERO);
        let mut scene = Scene::default();
        scene.add(&text.move_to(Vec2::new(8.0, 80.0)));
        target.render(&camera, &scene.bake());
        assert_eq!(target.readback().await.unwrap().data(), pixels);
    });
}
