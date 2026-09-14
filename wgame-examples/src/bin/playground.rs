//! Interactive lifecycle example: mouse tracking, pause, textures, text, resize,
//! animation, and a scoped background task. Run with --smoke for 12 frames.
use std::{cell::Cell, rc::Rc, time::Duration};
use wgame::{
    Event, Library, Result, Window,
    gfx::types::color,
    glam::Vec2,
    input::event::ElementState,
    input::keyboard::{KeyCode, PhysicalKey},
    prelude::*,
};

#[wgame::window(title = "wgame playground — mouse: move, space: pause", logical_size = (960.0,640.0))]
async fn main(mut window: Window<'_>) -> Result<()> {
    let library = Library::new(window.graphics());
    let font_data = wgame::typography::FontData::new(
        include_bytes!("../../assets/free-sans-bold.ttf").to_vec(),
        0,
    )?;
    let font = library.make_font(&font_data);
    let image = wgame::image::Image::decode_auto(include_bytes!("../../assets/lenna.png"))?;
    let texture = library.make_texture(&image, wgame::texture::TextureSettings::linear());
    let photo = library
        .shapes()
        .rectangle((Vec2::splat(-48.0), Vec2::splat(48.0)))
        .fill_texture(&texture);
    let ring = library
        .shapes()
        .unit_circle()
        .stroke_color(0.15, color::CYAN)
        .scale(35.0);
    let ticks = Rc::new(Cell::new(0usize));
    let counter = ticks.clone();
    let background = wgame::spawn(async move {
        loop {
            wgame::sleep(Duration::from_millis(250)).await;
            counter.set(counter.get() + 1);
        }
    })
    .cancel_on_drop();
    let mut input = window.input();
    let mut mouse = Vec2::new(200.0, 200.0);
    let mut paused = false;
    let mut angle = 0.0;
    let mut last = wgame::app::time::Instant::now();
    let mut scale_factor = window.scale_factor();
    let mut font_size = 22.0;
    let mut raster = font.rasterize(font_size * scale_factor as f32);
    let mut label = raster.text("Move the mouse. Space pauses animation.");
    let mut last_ticks = usize::MAX;
    #[cfg(not(target_arch = "wasm32"))]
    let smoke = std::env::args().any(|arg| arg == "--smoke");
    #[cfg(target_arch = "wasm32")]
    let smoke = false;
    let mut frames = 0;
    while let Some(mut frame) = window.next_frame().await? {
        while let Some(event) = input.try_next() {
            match event {
                Event::CursorMoved { position, .. } => {
                    mouse = Vec2::new(position.x as f32, position.y as f32)
                }
                Event::KeyboardInput { event, .. }
                    if event.state == ElementState::Pressed
                        && !event.repeat
                        && event.physical_key == PhysicalKey::Code(KeyCode::Space) =>
                {
                    paused = !paused;
                    last_ticks = usize::MAX;
                }
                _ => (),
            }
        }
        let now = wgame::app::time::Instant::now();
        if !paused {
            angle += (now - last).as_secs_f32().min(0.1);
        }
        last = now;
        let (width, height) = frame.logical_size();
        if frame.resized().is_some() || frame.scale_factor() != scale_factor {
            scale_factor = frame.scale_factor();
            font_size = (height as f32 / 28.0).clamp(14.0, 32.0);
            raster = font.rasterize(font_size * scale_factor as f32);
            last_ticks = usize::MAX;
        }
        if last_ticks != ticks.get() {
            label = raster.text(&format!(
                "Space: {} | background ticks: {}",
                if paused { "resume" } else { "pause" },
                ticks.get()
            ));
            last_ticks = ticks.get();
        }
        frame.clear(color::BLACK);
        let camera = frame.logical_camera();
        let pointer = camera.screen_to_world(mouse, frame.size());
        let mut scene = frame.scene();
        scene.camera = camera;
        scene.add(
            &photo
                .transform(wgame::glam::Affine2::from_angle(angle))
                .move_to(Vec2::new(width as f32 * 0.5, height as f32 * 0.5)),
        );
        if let Some(pointer) = pointer {
            scene.add(&ring.move_to(pointer));
        }
        scene.add(
            &label
                .scale(font_size)
                .move_to(Vec2::new(16.0, font_size + 12.0))
                .order(1),
        );
        scene.render();
        frame.present();
        frames += 1;
        if smoke && frames == 12 {
            break;
        }
    }
    background.handle().terminate();
    let _ = background.await;
    Ok(())
}
