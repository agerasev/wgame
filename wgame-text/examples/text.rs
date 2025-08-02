extern crate std;

use std::{
    fs::File,
    io::{Read, Write},
};

use wgame_text::{Font, RasterizedFont, Text};

fn main() {
    let mut contents = Vec::new();
    File::open("../wgame-examples/assets/free-sans-bold.ttf")
        .unwrap()
        .read_to_end(&mut contents)
        .unwrap();
    let font = Font::new(contents, 0).unwrap();
    let raster = RasterizedFont::new(&font, 64.0);
    let text = Text::new(&raster, "Hello, World!");
}
