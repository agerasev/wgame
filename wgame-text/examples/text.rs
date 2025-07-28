extern crate std;

use std::{
    fs::File,
    io::{Read, Write},
};

use wgame_text::{Font, FontAtlas, Text};

fn main() {
    let mut contents = Vec::new();
    File::open("../wgame-examples/assets/free-sans-bold.ttf")
        .unwrap()
        .read_to_end(&mut contents)
        .unwrap();
    let font = Font::new(contents, 0).unwrap();
    let atlas = FontAtlas::new(&font, 64.0);
    let text = Text::new(&atlas, "Hello, World!");
}
