//! Self-contained 3D gallery: geometry, lighting, maps, paint and custom shaders.
use wgame::{
    Library, Result, Window,
    canvas::{Event, Key},
    gfx::{Camera, DepthMode, Scene, Target, types::Color},
    glam::{Affine3A, Vec2, Vec3, Vec4, camera},
    image::Image,
    prelude::*,
    shapes::{
        Mesh, Polygon,
        material::{MeshMaterial, MeshShader},
        shader::{ShaderConfig, Vertex},
    },
    shapes3d::{
        AlbedoMode, LightParameters, Lighting, MaterialSettings, NormalSpace, Shapes3d,
        recalculate_normals,
    },
    texture::{Texture, TextureSettings},
};
use wgame_examples::{Labels, gallery};

struct Gallery {
    sphere: Polygon,
    cylinder: Polygon,
    pyramid: Polygon,
    base: Texture,
    tangent: Texture,
    object: Texture,
    lighting: Lighting,
    stripes: MeshMaterial<()>,
}
impl Gallery {
    fn new(lib: &Library) -> Result<Self> {
        // Duplicate face corners to retain a hard edge at each pyramid face.
        let points = [
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(1.0, -1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(-1.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.7),
        ];
        let mut vertices: Vec<_> = [
            [0, 1, 4],
            [1, 2, 4],
            [2, 3, 4],
            [3, 0, 4],
            [0, 3, 2],
            [0, 2, 1],
        ]
        .into_iter()
        .flatten()
        .map(|i| Vertex::new(points[i].extend(1.0), Vec3::Z))
        .collect();
        let indices: Vec<_> = (0..vertices.len() as u32).collect();
        recalculate_normals(&mut vertices, &indices)?;
        let pyramid = lib.shapes().mesh(Mesh::from_arrays(
            lib.shapes().state(),
            &vertices,
            Some(&indices),
        ));
        let image = |f: &dyn Fn(f32, f32) -> Vec4| {
            Image::with_data(
                (128, 64),
                (0..64)
                    .flat_map(|y| {
                        (0..128).map(move |x| {
                            f((x as f32 + 0.5) / 128.0, (y as f32 + 0.5) / 64.0).to_rgba_f16()
                        })
                    })
                    .collect::<Vec<_>>(),
            )
        };
        let base = lib.make_texture(
            &image(&|u, v| {
                let stripe = (u * 12.0).fract() < 0.5;
                let c = if (v - 0.5).abs() < 0.13 {
                    if stripe { 0.12 } else { 0.8 }
                } else {
                    0.4
                };
                Vec4::new(c, c, c, 1.0)
            }),
            TextureSettings::linear(),
        );
        let tangent = lib.make_texture(
            &image(&|u, v| {
                let n = Vec3::new(0.4 * (u * 50.0).sin(), 0.4 * (v * 40.0).cos(), 1.0).normalize();
                (n * 0.5 + Vec3::splat(0.5)).extend(if (v - 0.5).abs() < 0.13 { 0.0 } else { 1.0 })
            }),
            TextureSettings::linear(),
        );
        let object = lib.make_texture(
            &image(&|u, v| {
                let a = u * std::f32::consts::TAU;
                let b = v * std::f32::consts::PI;
                (Vec3::new(a.cos() * b.sin(), a.sin() * b.sin(), b.cos()) * 0.5 + Vec3::splat(0.5))
                    .extend(1.0)
            }),
            TextureSettings::linear(),
        );
        let shader = MeshShader::<()>::new(lib.shapes().state(), ShaderConfig {
            fragment_color_source: "color = vec4<f32>(mix(vec3<f32>(0.08, 0.18, 0.3), vec3<f32>(0.95, 0.5, 0.12), step(0.5, fract(input.local_coord.x * 12.0))), color.a);".into(),
            ..Default::default()
        }, &[])?;
        Ok(Self {
            sphere: lib.shapes().sphere(40, 24),
            cylinder: lib.shapes().cylinder(40),
            pyramid,
            base,
            tangent,
            object,
            lighting: Lighting::new(lib.shapes(), lib.texturing(), LightParameters::default())?,
            stripes: shader.material(vec![], ())?,
        })
    }
    fn scene(
        &self,
        lib: &Library,
        panel: usize,
        phase: f32,
        paint: Vec3,
        maps: bool,
    ) -> Result<Scene> {
        let mut scene = Scene::default();
        let settings = MaterialSettings {
            normal_strength: if maps { 1.0 } else { 0.0 },
            ..Default::default()
        };
        let matte = self.lighting.material(
            None,
            MaterialSettings {
                specular: 0.0,
                ..settings
            },
        )?;
        let glossy = self.lighting.material(
            None,
            MaterialSettings {
                specular: 0.65,
                shininess: 64.0,
                ..settings
            },
        )?;
        let ground = lib
            .shapes()
            .unit_quad()
            .transform(wgame::glam::Affine2::from_scale(Vec2::new(2.7, 1.8)))
            .fill_color(Vec4::new(0.09, 0.13, 0.18, 1.0))
            .with_material(&matte);
        scene.add(&ground);
        let rotation = Affine3A::from_rotation_z(phase * 0.45);
        match panel {
            0 => {
                scene.add(
                    &self
                        .sphere
                        .fill_color(Vec4::new(0.15, 0.5, 0.8, 1.0))
                        .with_material(&glossy)
                        .scale(0.6)
                        .move_to(Vec3::new(-1.5, 0.0, 0.6)),
                );
                scene.add(
                    &self
                        .cylinder
                        .fill_color(Vec4::new(0.8, 0.4, 0.08, 1.0))
                        .with_material(&matte)
                        .transform(Affine3A::from_scale(Vec3::new(0.55, 0.55, 1.3)))
                        .move_to(Vec3::new(0.0, 0.0, 0.65)),
                );
                scene.add(
                    &self
                        .pyramid
                        .fill_color(Vec4::new(0.35, 0.7, 0.25, 1.0))
                        .with_material(&matte)
                        .scale(0.65)
                        .transform(rotation)
                        .move_to(Vec3::new(1.5, 0.0, 0.0)),
                );
            }
            1 => {
                for (x, material) in [(-0.9, &matte), (0.9, &glossy)] {
                    scene.add(
                        &self
                            .sphere
                            .fill_color(Vec4::new(0.18, 0.48, 0.65, 1.0))
                            .with_material(material)
                            .scale(0.85)
                            .move_to(Vec3::new(x, 0.0, 0.85)),
                    );
                }
            }
            2 => {
                let mapped = self.lighting.material(Some(&self.tangent), settings)?;
                for (x, material) in [(-0.9, &matte), (0.9, &mapped)] {
                    scene.add(
                        &self
                            .sphere
                            .fill_texture(&self.base)
                            .with_material(material)
                            .scale(0.85)
                            .transform(rotation)
                            .move_to(Vec3::new(x, 0.0, 0.85)),
                    );
                }
            }
            3 => {
                let mapped = self.lighting.material(
                    Some(&self.object),
                    MaterialSettings {
                        normal_space: NormalSpace::Object,
                        ..settings
                    },
                )?;
                for (x, sx) in [(-1.0, 1.0), (1.0, -1.0)] {
                    scene.add(
                        &self
                            .sphere
                            .fill_color(Vec4::new(0.8, 0.3, 0.12, 1.0))
                            .with_material(&mapped)
                            .transform(Affine3A::from_scale(Vec3::new(sx * 0.65, 0.65, 1.1)))
                            .transform(rotation)
                            .move_to(Vec3::new(x, 0.0, 1.1)),
                    );
                }
            }
            4 => {
                let mapped = self.lighting.material(
                    Some(&self.tangent),
                    MaterialSettings {
                        albedo_mode: AlbedoMode::MaskedPaint,
                        ..settings
                    },
                )?;
                for (x, c) in [(-1.0, paint), (1.0, Vec3::new(paint.z, paint.x, paint.y))] {
                    scene.add(
                        &self
                            .sphere
                            .fill_texture(&self.base)
                            .with_material(&mapped)
                            .multiply_color(c.extend(1.0))
                            .scale(0.85)
                            .transform(rotation)
                            .move_to(Vec3::new(x, 0.0, 0.85)),
                    );
                }
            }
            _ => {
                scene.add(
                    &self
                        .sphere
                        .fill_color(Vec4::ONE)
                        .with_material(&self.stripes)
                        .scale(0.9)
                        .transform(rotation)
                        .move_to(Vec3::new(-0.6, 0.1, 0.9)),
                );
                scene.add(
                    &self
                        .cylinder
                        .fill_color(Vec4::new(0.15, 0.75, 0.5, 1.0))
                        .transform(Affine3A::from_scale(Vec3::new(0.5, 0.5, 1.6)))
                        .move_to(Vec3::new(1.0, 0.2, 0.8)),
                );
                // Draw transparent surfaces after opaque geometry, with depth read only.
                scene.add(
                    &lib.shapes()
                        .unit_quad()
                        .fill_color(Vec4::new(0.2, 0.6, 0.9, 0.25))
                        .transform(wgame::glam::Affine2::from_scale(Vec2::new(2.0, 0.75)))
                        .transform(Affine3A::from_rotation_x(std::f32::consts::FRAC_PI_2))
                        .move_to(Vec3::new(0.0, -0.7, 0.9))
                        .depth(DepthMode::ReadOnly),
                );
            }
        }
        Ok(scene)
    }
}

#[derive(Clone, Copy)]
struct ViewSettings {
    phase: f32,
    paint: Vec3,
    maps: bool,
    ortho: bool,
}
impl Gallery {
    fn draw(
        &self,
        target: &mut impl Target,
        lib: &Library,
        labels: &mut Labels,
        view: ViewSettings,
    ) -> Result<()> {
        let ViewSettings {
            phase,
            paint,
            maps,
            ortho,
        } = view;
        let eye = Vec3::new(1.5, -4.0, 2.5);
        self.lighting.update(LightParameters {
            eye,
            direction: Vec3::new(phase.cos(), phase.sin(), 1.5),
            ..Default::default()
        })?;
        let (camera2d, scale) = gallery::camera(target);
        labels.set_scale(scale);
        let size = Vec2::new(target.size().0 as f32, target.size().1 as f32);
        let offset = (size - gallery::SIZE * scale) * 0.5;
        target.clear(gallery::BACKGROUND);
        let mut ui = Scene::default();
        gallery::heading(
            &mut ui,
            labels,
            "3D materials",
            "Space: pause   P: paint color   M: normal maps   O: projection   Esc: close",
        );
        let panels = [
            (
                "Primitives + custom mesh",
                "Smooth sphere, capped cylinder, flat-face pyramid",
            ),
            (
                "Ambient + moving sun",
                "Matte versus glossy Blinn-Phong highlights",
            ),
            (
                "Tangent-space normals",
                "Same texture: geometry normals versus mapped relief",
            ),
            (
                "Object-space normals",
                "Nonuniform scales; right instance is reflected",
            ),
            (
                "Masked paint",
                "Shared textures, independent colors; trim stays fixed",
            ),
            (
                "Custom shader + depth",
                "WGSL stripes, unlit solid, translucent foreground",
            ),
        ];
        for (i, (title, detail)) in panels.into_iter().enumerate() {
            gallery::panel(&mut ui, lib, labels, i, title, detail);
        }
        target.render(&camera2d, &ui.bake());
        for panel in 0..6 {
            let origin = ((gallery::origin(panel) + Vec2::new(12.0, 68.0)) * scale + offset)
                .floor()
                .as_uvec2();
            let extent = (Vec2::new(516.0, 150.0) * scale).floor().as_uvec2();
            if extent.min_element() == 0 {
                continue;
            }
            let mut view = target.viewport((origin.x, origin.y), (extent.x, extent.y))?;
            view.clear(Vec4::new(0.035, 0.05, 0.075, 1.0));
            let aspect = extent.x as f32 / extent.y as f32;
            let projection = if ortho {
                camera::rh::proj::directx::orthographic(
                    -1.65 * aspect,
                    1.65 * aspect,
                    -1.65,
                    1.65,
                    0.1,
                    40.0,
                )
            } else {
                camera::rh::proj::directx::perspective(0.65, aspect, 0.1, 40.0)
            };
            let camera = Camera::new(
                lib.state(),
                projection * camera::rh::view::look_at_mat4(eye, Vec3::new(0.0, 0.0, 0.7), Vec3::Z),
            );
            view.render(&camera, &self.scene(lib, panel, phase, paint, maps)?.bake());
        }
        Ok(())
    }
}

#[wgame::window(title = "wgame: 3D materials", logical_size = (1200.0,900.0), resizable = true)]
async fn main(mut window: Window<'_>) -> Result<()> {
    let lib = Library::new(window.graphics());
    let gallery3d = Gallery::new(&lib)?;
    let mut labels = Labels::new(&lib)?;
    let mut paused = false;
    let mut maps = true;
    let mut ortho = false;
    let colors = [
        Vec3::new(0.8, 0.08, 0.025),
        Vec3::new(0.04, 0.25, 0.8),
        Vec3::new(0.9, 0.7, 0.07),
        Vec3::splat(0.025),
    ];
    let mut color_index = 0;
    let mut phase = 0.0;
    let mut last = wgame::app::time::Instant::now();
    let smoke = cfg!(not(target_arch = "wasm32")) && std::env::args().any(|arg| arg == "--smoke");
    let mut frames = 0;
    'frames: while let Some(mut frame) = window.next_frame().await? {
        for event in &frame.input().events {
            if let Event::Key {
                key,
                pressed: true,
                repeat: false,
            } = event
            {
                match key {
                    Key::Space => paused = !paused,
                    Key::Character('m') => maps = !maps,
                    Key::Character('p') => color_index = (color_index + 1) % colors.len(),
                    Key::Character('o') => ortho = !ortho,
                    Key::Escape => {
                        frame.discard();
                        break 'frames;
                    }
                    _ => {}
                }
            }
        }
        let now = wgame::app::time::Instant::now();
        if !paused {
            phase += (now - last).as_secs_f32().min(0.1);
        }
        last = now;
        gallery3d.draw(
            &mut frame,
            &lib,
            &mut labels,
            ViewSettings {
                phase,
                paint: colors[color_index],
                maps,
                ortho,
            },
        )?;
        frame.present();
        frames += 1;
        if smoke && frames == 12 {
            break;
        }
    }
    Ok(())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use wgame::image::{ImageRead, ImageReadExt};

    #[test]
    #[ignore = "requires a GPU adapter; run with --ignored"]
    fn gallery_panels_render_in_both_projections() {
        futures::executor::block_on(async {
            let instance = wgpu::Instance::new(
                wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
            );
            let adapter = instance
                .request_adapter(&Default::default())
                .await
                .expect("GPU adapter required");
            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                    ..Default::default()
                })
                .await
                .unwrap();
            let gfx =
                wgame::gfx::Graphics::new(adapter, device, queue, wgpu::TextureFormat::Rgba8Unorm);
            let lib = Library::new(&gfx);
            let gallery = Gallery::new(&lib).unwrap();
            let mut labels = Labels::new(&lib).unwrap();
            let mut target = lib
                .texturing()
                .render_texture((1200, 900), Default::default())
                .unwrap();
            let mut previous = None;
            for ortho in [false, true] {
                gallery
                    .draw(
                        &mut target,
                        &lib,
                        &mut labels,
                        ViewSettings {
                            phase: 0.5,
                            paint: Vec3::new(0.8, 0.08, 0.025),
                            maps: true,
                            ortho,
                        },
                    )
                    .unwrap();
                let image = target.readback().await.unwrap();
                for panel in 0..6 {
                    let origin = gallery::origin(panel) + Vec2::new(12.0, 68.0);
                    let mut bright = 0;
                    for y in origin.y as u32..origin.y as u32 + 150 {
                        for x in origin.x as u32..origin.x as u32 + 516 {
                            let p = image.get((x, y).into());
                            bright +=
                                usize::from(p.r.to_f32().max(p.g.to_f32()).max(p.b.to_f32()) > 0.2);
                        }
                    }
                    assert!(
                        bright > 1000,
                        "panel {panel} should contain visible geometry"
                    );
                }
                if let Some(pixels) = previous {
                    assert_ne!(
                        image.data(),
                        pixels,
                        "projection toggle should change the view"
                    );
                }
                previous = Some(image.data().to_vec());
                if let Some(dir) = std::env::var_os("WGAME_RENDER_OUTPUT") {
                    std::fs::write(
                        std::path::Path::new(&dir).join(format!(
                            "solids-{}.png",
                            if ortho { "orthographic" } else { "perspective" }
                        )),
                        image.slice((.., ..)).encode("png").unwrap(),
                    )
                    .unwrap();
                }
            }
        });
    }
}
