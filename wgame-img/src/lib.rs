#![forbid(unsafe_code)]

use std::io::Cursor;

use anyhow::Result;
use half::f16;
use image::ImageReader;

use wgame_shapes::{Library, Texture};

pub fn image_to_texture(state: &Library, bytes: &[u8]) -> Result<Texture> {
    let reader = Cursor::new(bytes);
    let image = ImageReader::new(reader).with_guessed_format()?.decode()?;

    let data: Vec<f16> = {
        // TODO: Convert directly to f16
        let mut image = image.to_rgba32f();
        // Convert to sRGB
        for pix in image.pixels_mut() {
            for ch in &mut pix.0[0..3] {
                *ch = ch.powf(2.2);
            }
        }
        image.into_vec().into_iter().map(f16::from_f32).collect()
    };

    Ok(state.texture_with_data((image.width(), image.height()), bytemuck::cast_slice(&data)))
}
