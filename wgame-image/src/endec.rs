use anyhow::{Error, Result};
use euclid::default::Size2D;
use half::f16;
use image::{GrayImage, ImageFormat, ImageReader, Rgba32FImage};
use rgb::{ComponentMap, Rgba};
use std::io::Cursor;

use crate::{Image, ImageBase, ImageReadExt, ImageSlice};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Encoding {
    #[cfg(feature = "png")]
    Png,
}

impl Into<ImageFormat> for Encoding {
    fn into(self) -> ImageFormat {
        match self {
            #[cfg(feature = "png")]
            Self::Png => ImageFormat::Png,
        }
    }
}

impl TryFrom<image::DynamicImage> for Image<Rgba<f16>> {
    type Error = Error;
    fn try_from(image: image::DynamicImage) -> Result<Self> {
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

        Ok(Self::with_data(
            (image.width(), image.height()),
            bytemuck::cast_slice(&data).to_vec(),
        ))
    }
}

impl Image<Rgba<f16>> {
    fn decode_inner(bytes: &[u8], encoding: Option<ImageFormat>) -> Result<Self> {
        let mut reader = ImageReader::new(Cursor::new(bytes));
        match encoding {
            None => reader = reader.with_guessed_format()?,
            Some(enc) => reader.set_format(enc),
        }

        let image = reader.decode()?;
        Self::try_from(image)
    }

    pub fn decode(bytes: &[u8], encoding: impl Into<ImageFormat>) -> Result<Self> {
        Self::decode_inner(bytes, Some(encoding.into()))
    }

    pub fn decode_auto(bytes: &[u8]) -> Result<Self> {
        Self::decode_inner(bytes, None)
    }
}

impl ImageSlice<'_, Rgba<f16>> {
    pub fn encode(&self, encoding: impl Into<ImageFormat>) -> Result<Vec<u8>> {
        let Size2D { width, height, .. } = self.size();
        let data: Vec<Rgba<f32>> = self
            .rows()
            .flat_map(|(_, row)| row.iter().map(|c| c.map(|v| f32::from(v).powf(1.0 / 2.2))))
            .collect();
        let image = Rgba32FImage::from_vec(width, height, bytemuck::cast_vec(data))
            .expect("Buffer is smaller than expected");

        let mut buffer = Cursor::new(Vec::<u8>::new());
        image.write_to(&mut buffer, encoding.into())?;
        Ok(buffer.into_inner())
    }
}

impl ImageSlice<'_, u8> {
    pub fn encode(&self, encoding: impl Into<ImageFormat>) -> Result<Vec<u8>> {
        let Size2D { width, height, .. } = self.size();
        let data: Vec<u8> = self
            .rows()
            .flat_map(|(_, row)| row.iter().copied())
            .collect();
        let image = GrayImage::from_vec(width, height, bytemuck::cast_vec(data))
            .expect("Buffer is smaller than expected");

        let mut buffer = Cursor::new(Vec::<u8>::new());
        image.write_to(&mut buffer, encoding.into())?;
        Ok(buffer.into_inner())
    }
}
