#![forbid(unsafe_code)]

use std::io::Cursor;

use anyhow::{Result, bail};
use half::f16;
use image::ImageReader;
use rgb::Rgba;

pub struct Image {
    size: (u32, u32),
    data: Vec<Rgba<f16>>,
}

impl Image {
    pub fn new(size: impl Into<(u32, u32)>, data: impl Into<Vec<Rgba<f16>>>) -> Result<Self> {
        let this = Self {
            size: size.into(),
            data: data.into(),
        };
        if (this.size.0 * this.size.1) as usize != this.data.len() {
            bail!(
                "Image size ({:?}) and data length {:?} do not match",
                this.size,
                this.data.len()
            );
        }
        Ok(this)
    }

    pub fn from_formatted_data(bytes: &[u8]) -> Result<Self> {
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

        Self::new((image.width(), image.height()), bytemuck::cast_slice(&data))
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }
    pub fn data(&self) -> &[Rgba<f16>] {
        &self.data
    }
}
