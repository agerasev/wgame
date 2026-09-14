//! Four-point quads and variable-width ribbons. Space pauses the angle sweep;
//! P toggles junction markers. Run with --smoke for twelve frames.
use wgame::{
    Event, Library, Result, Window,
    gfx::{
        Scene,
        types::{Color, color},
    },
    glam::{Affine2, Vec2},
    image::Image,
    input::keyboard::{KeyCode, PhysicalKey},
    prelude::*,
    rgb::Rgba,
    shapes::{PolylinePoint, ShapesLibrary},
    texture::{Texture, TextureSettings},
};

const SIZE: Vec2 = Vec2::new(1200.0, 900.0);

fn grid_texture(library: &Library) -> Texture {
    let pixels = (0..64)
        .flat_map(|y| {
            (0..256).map(move |x| {
                if x % 32 == 0 || y % 16 == 0 {
                    color::WHITE.to_rgba_f16()
                } else {
                    let u = x as f32 / 255.0;
                    let v = y as f32 / 63.0;
                    Rgba::new(0.15 + 0.8 * u, 0.7 - 0.4 * u, 0.9 - 0.5 * v, 1.0).to_rgba_f16()
                }
            })
        })
        .collect::<Vec<_>>();
    library.make_texture(
        &Image::with_data((256, 64), pixels),
        TextureSettings::nearest(),
    )
}

fn ribbon(
    shapes: &ShapesLibrary,
    scene: &mut Scene,
    origin: Vec2,
    points: &[(f32, f32, f32)],
    texture: Option<&Texture>,
    markers: bool,
) {
    let points: Vec<_> = points
        .iter()
        .map(|&(x, y, width)| PolylinePoint {
            position: origin + Vec2::new(x, y),
            width,
        })
        .collect();
    let shape = shapes.polyline(&points);
    if let Some(texture) = texture {
        scene.add(&shape.fill_texture(texture));
    } else {
        scene.add(&shape.fill_color(Rgba::new(0.15, 0.85, 1.0, 0.5)));
    }
    if markers {
        for pair in points.windows(2) {
            scene.add(
                &shapes
                    .line(pair[0].position, pair[1].position, 1.0)
                    .fill_color(Rgba::new(1.0, 1.0, 1.0, 0.6)),
            );
        }
        for point in points {
            scene.add(
                &shapes
                    .unit_circle()
                    .scale(3.0)
                    .move_to(point.position)
                    .fill_color(color::WHITE),
            );
        }
    }
}

#[wgame::window(title = "wgame: quads and polylines", size = (1200, 900), resizable = true, vsync = true)]
async fn main(mut window: Window<'_>) -> Result<()> {
    let library = Library::new(window.graphics());
    let shapes = library.shapes();
    let texture = grid_texture(&library);
    let font = library.make_font(&wgame::typography::FontData::new(
        include_bytes!("../../assets/free-sans-bold.ttf").to_vec(),
        0,
    )?);
    let raster = font.rasterize(20.0);
    let captions = [
        ("Quads and polylines", 30.0, 37.0, 30.0),
        (
            "Space: pause angle sweep    P: junction markers    Esc: close",
            30.0,
            70.0,
            19.0,
        ),
        ("Four-point quad", 48.0, 123.0, 22.0),
        (
            "Same grid on a rectangle and a skewed quad",
            48.0,
            150.0,
            17.0,
        ),
        ("Variable widths", 648.0, 123.0, 22.0),
        (
            "Flat start, shared miters, taper to zero",
            648.0,
            150.0,
            17.0,
        ),
        ("Miter limit 4", 48.0, 383.0, 22.0),
        (
            "Bevel below an inside angle of about 29 degrees",
            48.0,
            410.0,
            17.0,
        ),
        ("Texture progress follows point index", 648.0, 383.0, 22.0),
        (
            "The short first segment uses half the texture",
            648.0,
            410.0,
            17.0,
        ),
        ("Repeated points and zero width", 48.0, 643.0, 22.0),
        (
            "Repeated start keeps its last width; middle pinches",
            48.0,
            670.0,
            17.0,
        ),
        ("Transparency and self-intersections", 648.0, 643.0, 22.0),
        (
            "50% opacity: crossings and reversed runs overlap",
            648.0,
            670.0,
            17.0,
        ),
        (
            "UVs interpolate within each triangle; grid lines can kink at the quad diagonal.",
            30.0,
            879.0,
            17.0,
        ),
    ]
    .map(|(text, x, y, size)| raster.text(text).scale(size).move_to(Vec2::new(x, y)));

    let mut input = window.input();
    let mut paused = false;
    let mut markers = true;
    let mut time = 0.0_f32;
    let mut last = wgame::app::time::Instant::now();
    #[cfg(not(target_arch = "wasm32"))]
    let smoke = std::env::args().any(|arg| arg == "--smoke");
    #[cfg(target_arch = "wasm32")]
    let smoke = false;
    let mut frames = 0;
    'frames: while let Some(mut frame) = window.next_frame().await? {
        while let Some(event) = input.try_next() {
            if let Event::KeyboardInput { event, .. } = event
                && event.state.is_pressed()
                && !event.repeat
            {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Space) => paused = !paused,
                    PhysicalKey::Code(KeyCode::KeyP) => markers = !markers,
                    PhysicalKey::Code(KeyCode::Escape) => {
                        frame.discard();
                        break 'frames;
                    }
                    _ => (),
                }
            }
        }
        let now = wgame::app::time::Instant::now();
        if !paused {
            time += (now - last).as_secs_f32().min(0.1);
        }
        last = now;
        let angle = 26.0 + 14.0 * (time * 0.6).sin();
        let viewport = Vec2::new(frame.size().0 as f32, frame.size().1 as f32);
        let scale = (viewport / SIZE).min_element();
        let camera = frame
            .physical_camera()
            .transform(Affine2::from_scale_angle_translation(
                Vec2::splat(scale),
                0.0,
                (viewport - scale * SIZE) * 0.5,
            ));
        frame.clear(Rgba::new(0.025, 0.035, 0.055, 1.0));
        let mut scene = frame.scene();
        scene.camera = camera;
        for y in [100.0, 360.0, 620.0] {
            for x in [30.0, 630.0] {
                scene.add(
                    &shapes
                        .rectangle((Vec2::new(x, y), Vec2::new(x + 540.0, y + 235.0)))
                        .fill_color(Rgba::new(0.06, 0.08, 0.11, 1.0)),
                );
            }
        }
        for caption in &captions {
            scene.add(caption);
        }

        scene.add(
            &shapes
                .rectangle((Vec2::new(55.0, 185.0), Vec2::new(245.0, 310.0)))
                .fill_texture(&texture),
        );
        let quad = [
            Vec2::new(285.0, 175.0),
            Vec2::new(540.0, 205.0),
            Vec2::new(490.0, 320.0),
            Vec2::new(315.0, 285.0),
        ];
        scene.add(
            &shapes
                .quad(quad[0], quad[1], quad[2], quad[3])
                .fill_texture(&texture),
        );
        if markers {
            scene.add(&shapes.line(quad[0], quad[2], 1.0).fill_color(color::WHITE));
        }
        ribbon(
            shapes,
            &mut scene,
            Vec2::new(650.0, 180.0),
            &[
                (20.0, 85.0, 55.0),
                (140.0, 30.0, 42.0),
                (290.0, 100.0, 24.0),
                (485.0, 25.0, 0.0),
            ],
            Some(&texture),
            markers,
        );

        let theta = angle.to_radians();
        ribbon(
            shapes,
            &mut scene,
            Vec2::new(60.0, 425.0),
            &[
                (20.0, 125.0, 22.0),
                (430.0, 125.0, 22.0),
                (
                    430.0 - 190.0 * theta.cos(),
                    125.0 - 190.0 * theta.sin(),
                    22.0,
                ),
            ],
            None,
            markers,
        );
        scene.add(
            &raster
                .text(&format!(
                    "{angle:.1} degrees  |  {}{}",
                    if angle < 2.0 * 0.25_f32.asin().to_degrees() {
                        "bevel"
                    } else {
                        "miter"
                    },
                    if paused { "  (paused)" } else { "" }
                ))
                .scale(18.0)
                .move_to(Vec2::new(60.0, 590.0)),
        );
        ribbon(
            shapes,
            &mut scene,
            Vec2::new(650.0, 445.0),
            &[(20.0, 80.0, 50.0), (90.0, 80.0, 50.0), (480.0, 20.0, 35.0)],
            Some(&texture),
            markers,
        );

        ribbon(
            shapes,
            &mut scene,
            Vec2::new(50.0, 700.0),
            &[
                (20.0, 75.0, 8.0),
                (20.0, 75.0, 45.0),
                (170.0, 25.0, 22.0),
                (300.0, 75.0, 0.0),
                (485.0, 25.0, 45.0),
            ],
            None,
            markers,
        );
        ribbon(
            shapes,
            &mut scene,
            Vec2::new(650.0, 700.0),
            &[
                (20.0, 20.0, 18.0),
                (250.0, 110.0, 18.0),
                (250.0, 20.0, 18.0),
                (20.0, 110.0, 18.0),
            ],
            None,
            markers,
        );
        ribbon(
            shapes,
            &mut scene,
            Vec2::new(650.0, 700.0),
            &[
                (340.0, 25.0, 18.0),
                (490.0, 90.0, 18.0),
                (340.0, 25.0, 18.0),
            ],
            None,
            markers,
        );
        scene.render();
        frame.present();
        frames += 1;
        if smoke && frames == 12 {
            break;
        }
    }
    Ok(())
}
