use std::io::Cursor;

use anyhow::Result;
use euclid::default::{Rect, Size2D};
use half::f16;
use image::ImageReader;
use rgb::Rgba;

use crate::{ImageBase, ImageRead, ImageReadExt, ImageResize, ImageWrite, ImageWriteMut, Pixel};

pub struct Image<P: Pixel> {
    size: Size2D<u32>,
    data: Vec<P>,
}

pub struct ImageSlice<'a, P: Pixel> {
    pub(crate) size: Size2D<u32>,
    pub(crate) stride: u32,
    pub(crate) data: &'a [P],
}

pub struct ImageSliceMut<'a, P: Pixel> {
    pub(crate) size: Size2D<u32>,
    pub(crate) stride: u32,
    pub(crate) data: &'a mut [P],
}

impl<P: Pixel> Image<P> {
    pub fn new(size: impl Into<Size2D<u32>>, data: impl Into<Vec<P>>) -> Self {
        let this = Self {
            size: size.into(),
            data: data.into(),
        };
        assert_eq!(
            this.size.width as u64 * this.size.height as u64,
            this.data.len() as u64,
            "Image size ({:?}) and data length {:?} do not match",
            this.size,
            this.data.len()
        );
        this
    }
}

impl<P: Pixel> ImageBase for Image<P> {
    type Pixel = P;

    fn size(&self) -> Size2D<u32> {
        self.size
    }
}

impl<P: Pixel> ImageRead for Image<P> {
    fn stride(&self) -> u32 {
        self.size.width
    }
    fn data(&self) -> &[P] {
        &self.data
    }
}

impl<P: Pixel> ImageWrite for Image<P> {
    fn data_mut(&mut self) -> &mut [P] {
        &mut self.data
    }
}

impl<P: Pixel> ImageBase for ImageSlice<'_, P> {
    type Pixel = P;

    fn size(&self) -> Size2D<u32> {
        self.size
    }
}

impl<P: Pixel> ImageRead for ImageSlice<'_, P> {
    fn stride(&self) -> u32 {
        self.stride
    }
    fn data(&self) -> &[P] {
        self.data
    }
}

impl<P: Pixel> ImageBase for ImageSliceMut<'_, P> {
    type Pixel = P;

    fn size(&self) -> Size2D<u32> {
        self.size
    }
}

impl<P: Pixel> ImageRead for ImageSliceMut<'_, P> {
    fn stride(&self) -> u32 {
        self.stride
    }
    fn data(&self) -> &[P] {
        self.data
    }
}

impl<P: Pixel> ImageWrite for ImageSliceMut<'_, P> {
    fn data_mut(&mut self) -> &mut [P] {
        self.data
    }
}

impl Image<Rgba<f16>> {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
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

        Ok(Self::new(
            (image.width(), image.height()),
            bytemuck::cast_slice(&data).to_vec(),
        ))
    }
}

impl<P: Pixel> ImageResize for Image<P> {
    fn resize(&mut self, new_size: impl Into<Size2D<u32>>, fill: P) {
        let new_size = new_size.into();
        if self.size == new_size {
            return;
        }
        let mut new_image = Image::new(new_size, vec![fill; new_size.cast::<usize>().area()]);
        let common_rect = Rect::from_size(new_size.min(self.size));
        new_image
            .slice_mut(common_rect)
            .copy_from(self.slice(common_rect));
        *self = new_image
    }
}
