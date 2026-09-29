use std::{fs::File, io::Write};

use wgame_image::{Atlas, Encoding};
use wgame_typography::{Font, FontAtlas};

fn main() {
    std::fs::create_dir_all("output").unwrap();
    let contents = include_bytes!("assets/free-sans-bold.ttf").to_vec();
    let font = Font::new(contents, 0).unwrap();
    let atlas = FontAtlas::new(&Atlas::default(), &font, 64.0);
    atlas.add_chars("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz01234567890".chars());

    atlas.image().with(|img| {
        let mut file = File::create("output/atlas.png").unwrap();
        file.write_all(&img.encode(Encoding::Png).unwrap()).unwrap();
    });
    File::create("output/atlas.svg")
        .unwrap()
        .write_all(&atlas.atlas_svg())
        .unwrap();
}
