#![forbid(unsafe_code)]

use std::io::Cursor;

use anyhow::Result;
use half::f16;
use image::ImageReader;

use wgame_gfx::{State, Texture};

pub fn image_to_texture<'a>(state: &State<'a>, bytes: &[u8]) -> Result<Texture<'a>> {
    let reader = Cursor::new(bytes);
    let image = ImageReader::new(reader).with_guessed_format()?.decode()?;

    // TODO: Convert directly to f16
    let data: Vec<f16> = image
        .to_rgba32f()
        .into_vec()
        .into_iter()
        .map(f16::from_f32)
        .collect();

    Ok(Texture::with_data(
        state,
        (image.width(), image.height()),
        bytemuck::cast_slice(&data),
    ))
}
