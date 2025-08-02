extern crate std;

use std::{
    fs::File,
    io::{Read, Write},
};

use wgame_text::{Font, RasterizedFont};

fn main() {
    let mut contents = Vec::new();
    File::open("../wgame-examples/assets/free-sans-bold.ttf")
        .unwrap()
        .read_to_end(&mut contents)
        .unwrap();
    let font = Font::new(contents, 0).unwrap();
    let atlas = RasterizedFont::new(&font, 64.0);
    atlas.add_chars("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz01234567890".chars());

    atlas.image().save("output/atlas.png").unwrap();
    File::create("output/atlas.svg")
        .unwrap()
        .write_all(&atlas.atlas_svg())
        .unwrap();
}
