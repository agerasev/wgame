extern crate std;

use rand::{
    Rng, SeedableRng,
    distr::{Bernoulli, Uniform},
    rngs::SmallRng,
    seq::IteratorRandom,
};
use std::{
    fs::File,
    io::{Read, Write},
};
use wgame_font::{Font, FontAtlas};
use wgame_image::{Atlas, Encoding, ImageReadExt};

const CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz01234567890";

fn main() {
    let mut contents = Vec::new();
    File::open("../wgame-examples/assets/free-sans-bold.ttf")
        .unwrap()
        .read_to_end(&mut contents)
        .unwrap();
    let font = Font::new(contents, 0).unwrap();

    let mut rng = SmallRng::seed_from_u64(0xdeadbeef);
    let atlas = Atlas::default();
    let mut styles = Vec::new();

    for _ in 0..100 {
        if rng.sample(Bernoulli::new(0.8).unwrap()) {
            let size = (rng
                .sample::<f32, _>(Uniform::new(0.0, 1.0).unwrap())
                .powi(2)
                * 100.0)
                .max(1.0);
            let style = FontAtlas::new(&atlas, &font, size);
            let mut chars = (0..(rng.sample(Uniform::new(0, CHARS.len()).unwrap())) + 1)
                .map(|_| char::default())
                .collect::<Vec<_>>();
            CHARS.chars().choose_multiple_fill(&mut rng, &mut chars);
            style.add_chars(chars);
            styles.push(style);
        } else {
            styles.remove(rng.sample(Uniform::new(0, styles.len()).unwrap()));
        }
    }

    atlas.with_data(|img| {
        let mut file = File::create("output/random.png").unwrap();
        file.write_all(&img.slice((.., ..)).encode(Encoding::Png).unwrap())
            .unwrap();
    });
}
