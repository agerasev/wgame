use image::GrayImage;
use swash::{
    FontRef,
    scale::{Render, ScaleContext, Source, StrikeWith},
};

fn main() {
    std::fs::create_dir_all("output").unwrap();
    let contents = include_bytes!("assets/free-sans-bold.ttf").to_vec();
    let font = FontRef::from_index(&contents, 0).expect("Font data validation error");
    let mut context = ScaleContext::new();
    let mut scaler = context.builder(font).size(64.0).hint(false).build();
    let render = Render::new(&[
        Source::ColorOutline(0),
        Source::ColorBitmap(StrikeWith::BestFit),
        Source::Outline,
    ]);
    let glyph_id = font.charmap().map('g');
    let image = render.render(&mut scaler, glyph_id).unwrap();
    dbg!(image.content);
    dbg!(image.source);
    dbg!(image.placement);
    dbg!(image.data.len());

    GrayImage::from_vec(image.placement.width, image.placement.height, image.data)
        .unwrap()
        .save("output/glyph.png")
        .unwrap()
}
