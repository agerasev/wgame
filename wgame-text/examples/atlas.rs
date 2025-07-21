extern crate std;

use std::{
    fs::File,
    io::{Read, Write},
};

use wgame_text::{Font, FontAtlas};

fn main() {
    let mut contents = Vec::new();
    File::open("../wgame-examples/assets/free-sans-bold.ttf")
        .unwrap()
        .read_to_end(&mut contents)
        .unwrap();
    let mut font = Font::new(contents).unwrap();
    let mut atlas = FontAtlas::new(&mut font, 64.0);
    for c in "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz01234567890".chars() {
        atlas.add_char(c);
    }

    atlas.image().save("output/atlas.png").unwrap();
    File::create("output/atlas.svg")
        .unwrap()
        .write_all(&atlas.debug_svg())
        .unwrap();
}
