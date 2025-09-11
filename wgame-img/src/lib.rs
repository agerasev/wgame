#![forbid(unsafe_code)]

use std::{
    borrow::Cow,
    io::Cursor,
    ops::{Bound, Range, RangeBounds},
};

use anyhow::{Result, bail};
use bytemuck::Pod;
use half::f16;
use image::ImageReader;
use rgb::Rgba;

pub trait Pixel: Pod {}

impl Pixel for u8 {}
impl Pixel for Rgba<f16> {}

pub trait ImageLike<T: Pixel> {
    fn size(&self) -> (u32, u32);
    fn stride(&self) -> (u32, u32);
    fn data(&self) -> &[T];
}

pub struct Image<T: Pixel = Rgba<f16>> {
    size: (u32, u32),
    data: Vec<T>,
}

pub struct ImageSlice<'a, T: Pixel> {
    size: (u32, u32),
    stride: (u32, u32),
    data: &'a [T],
}

impl<T: Pixel> Image<T> {
    pub fn new(size: impl Into<(u32, u32)>, data: impl Into<Vec<T>>) -> Self {
        let this = Self {
            size: size.into(),
            data: data.into(),
        };
        assert_eq!(
            this.size.0 as u64 * this.size.1 as u64,
            this.data.len() as u64,
            "Image size ({:?}) and data length {:?} do not match",
            this.size,
            this.data.len()
        );
        this
    }
}

impl<T: Pixel> ImageLike<T> for Image<T> {
    fn size(&self) -> (u32, u32) {
        self.size
    }
    fn stride(&self) -> (u32, u32) {
        (1, self.size.0)
    }
    fn data(&self) -> &[T] {
        &self.data
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

fn into_range(range: impl RangeBounds<u32>, size: u32) -> Range<u32> {
    let start = match range.start_bound() {
        Bound::Included(x) => *x,
        Bound::Excluded(_) => unimplemented!("Excluded start bound is not supported"),
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(x) => *x + 1,
        Bound::Excluded(x) => *x,
        Bound::Unbounded => size,
    };
    assert!((0..size).contains(&start));
    assert!((1..=size).contains(&end));
    assert!(start <= end);
    start..end
}

pub trait ImageLikeExt<T: Pixel>: ImageLike<T> {
    fn slice(
        &self,
        x_range: impl RangeBounds<u32>,
        y_range: impl RangeBounds<u32>,
    ) -> ImageSlice<'_, T> {
        let size = self.size();
        let x_range = into_range(x_range, size.0);
        let y_range = into_range(y_range, size.1);

        let new_size = (x_range.end - x_range.start, y_range.end - y_range.start);
        let stride = self.stride();
        ImageSlice {
            size: new_size,
            stride: (stride.0, stride.1 + (size.0 - new_size.0) * stride.0),
            data: &self.data()[unimplemented!()],
        }
    }
}

impl<T: Pixel, I: ImageLike<T>> ImageLikeExt<T> for I {}
