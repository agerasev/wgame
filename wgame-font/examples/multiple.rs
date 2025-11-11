use std::{
    fs::File,
    io::{Read, Write},
};

use wgame_font::{Font, FontAtlas};
use wgame_image::{Atlas, Encoding, ImageReadExt};

const CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz01234567890";
const SIZES: &[f32] = &[7.0, 64.0, 49.0, 8.0, 13.0, 27.0, 36.0, 1.0, 24.0];

fn main() {
    let mut contents = Vec::new();
    File::open("../wgame-examples/assets/free-sans-bold.ttf")
        .unwrap()
        .read_to_end(&mut contents)
        .unwrap();
    let font = Font::new(contents, 0).unwrap();

    let atlas = Atlas::default();
    let mut styles = Vec::new();
    for size in SIZES {
        let style = FontAtlas::new(&atlas, &font, *size);
        style.add_chars(CHARS.chars());
        styles.push(style);
    }

    atlas.with_data(|img| {
        let mut file = File::create("output/multiple.png").unwrap();
        file.write_all(&img.slice((.., ..)).encode(Encoding::Png).unwrap())
            .unwrap();
    });
}
