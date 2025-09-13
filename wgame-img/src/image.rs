use std::{
    io::Cursor,
    ops::{Bound, Range, RangeBounds},
};

use anyhow::Result;
use half::f16;
use image::ImageReader;
use rgb::Rgba;

use crate::Pixel;

pub trait ImageLike<T: Pixel> {
    fn size(&self) -> (u32, u32);
    fn stride(&self) -> u32;
    fn data(&self) -> &[T];
}

pub trait ImageLikeMut<T: Pixel>: ImageLike<T> {
    fn data_mut(&mut self) -> &mut [T];
}

pub struct Image<T: Pixel = Rgba<f16>> {
    size: (u32, u32),
    data: Vec<T>,
}

pub struct ImageSlice<'a, T: Pixel> {
    size: (u32, u32),
    stride: u32,
    data: &'a [T],
}

pub struct ImageSliceMut<'a, T: Pixel> {
    size: (u32, u32),
    stride: u32,
    data: &'a mut [T],
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
    fn stride(&self) -> u32 {
        self.size.0
    }
    fn data(&self) -> &[T] {
        &self.data
    }
}

impl<T: Pixel> ImageLikeMut<T> for Image<T> {
    fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}

impl<T: Pixel> ImageLike<T> for ImageSlice<'_, T> {
    fn size(&self) -> (u32, u32) {
        self.size
    }
    fn stride(&self) -> u32 {
        self.stride
    }
    fn data(&self) -> &[T] {
        self.data
    }
}

impl<T: Pixel> ImageLike<T> for ImageSliceMut<'_, T> {
    fn size(&self) -> (u32, u32) {
        self.size
    }
    fn stride(&self) -> u32 {
        self.stride
    }
    fn data(&self) -> &[T] {
        self.data
    }
}

impl<T: Pixel> ImageLikeMut<T> for ImageSliceMut<'_, T> {
    fn data_mut(&mut self) -> &mut [T] {
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
    assert!((0..=size).contains(&end));
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
        let data_start = x_range.start as usize + self.stride() as usize * y_range.start as usize;
        let data_end = if x_range.is_empty() || y_range.is_empty() {
            data_start
        } else {
            (x_range.end - 1) as usize + self.stride() as usize * (y_range.end - 1) as usize + 1
        };
        ImageSlice {
            size: (x_range.end - x_range.start, y_range.end - y_range.start),
            stride: self.stride(),
            data: &self.data()[data_start..data_end],
        }
    }

    fn get(&self, i: u32, j: u32) -> &T {
        let size = self.size();
        let stride = self.stride();
        assert!(
            i < size.0 && j < size.1,
            "({i}, {j}) is out of bounds {size:?}"
        );
        &self.data()[i as usize + stride as usize * j as usize]
    }

    fn rows(&self) -> impl ExactSizeIterator<Item = (u32, &[T])> {
        let size = self.size();
        let stride = self.stride();
        self.data()
            .chunks(stride as usize)
            .enumerate()
            .map(move |(j, row)| (j as u32, row.split_at(size.0 as usize).0))
    }

    fn pixels(&self) -> impl Iterator<Item = ((u32, u32), &T)> {
        self.rows().flat_map(|(j, row)| {
            row.iter()
                .enumerate()
                .map(move |(i, pixel)| ((i as u32, j), pixel))
        })
    }
}

pub trait ImageLikeMutExt<T: Pixel>: ImageLikeMut<T> {
    fn slice_mut(
        &mut self,
        x_range: impl RangeBounds<u32>,
        y_range: impl RangeBounds<u32>,
    ) -> ImageSliceMut<'_, T> {
        let size = self.size();
        let x_range = into_range(x_range, size.0);
        let y_range = into_range(y_range, size.1);
        let data_start = x_range.start as usize + self.stride() as usize * y_range.start as usize;
        let data_end = if x_range.is_empty() || y_range.is_empty() {
            data_start
        } else {
            (x_range.end - 1) as usize + self.stride() as usize * (y_range.end - 1) as usize + 1
        };
        ImageSliceMut {
            size: (x_range.end - x_range.start, y_range.end - y_range.start),
            stride: self.stride(),
            data: &mut self.data_mut()[data_start..data_end],
        }
    }

    fn get_mut(&mut self, i: u32, j: u32) -> &mut T {
        let size = self.size();
        let stride = self.stride();
        assert!(
            i < size.0 && j < size.1,
            "({i}, {j}) is out of bounds {size:?}"
        );
        &mut self.data_mut()[i as usize + stride as usize * j as usize]
    }

    fn rows_mut(&mut self) -> impl ExactSizeIterator<Item = (u32, &mut [T])> {
        let size = self.size();
        let stride = self.stride();
        self.data_mut()
            .chunks_mut(stride as usize)
            .enumerate()
            .map(move |(j, row)| (j as u32, row.split_at_mut(size.0 as usize).0))
    }

    fn pixels_mut(&mut self) -> impl Iterator<Item = ((u32, u32), &mut T)> {
        self.rows_mut().flat_map(|(j, row)| {
            row.iter_mut()
                .enumerate()
                .map(move |(i, pixel)| ((i as u32, j), pixel))
        })
    }

    fn copy_from(&mut self, src: impl ImageLike<T>) {
        assert_eq!(self.size(), src.size());
        for ((_, dst), (_, src)) in self.rows_mut().zip(src.rows()) {
            dst.copy_from_slice(src);
        }
    }
}

impl<T: Pixel, I: ImageLike<T>> ImageLikeExt<T> for I {}
impl<T: Pixel, I: ImageLikeMut<T>> ImageLikeMutExt<T> for I {}
