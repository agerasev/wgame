use euclid::default::{Box2D, Point2D, Rect, Size2D};
use std::ops::{Bound, Range, RangeBounds};

use crate::{ImageSlice, ImageSliceMut, Pixel};

pub trait ImageBase {
    type Pixel: Pixel;
    fn size(&self) -> Size2D<u32>;
}

pub trait ImageRead: ImageBase {
    /// Offset between rows in pixels
    fn stride(&self) -> u32;
    fn data(&self) -> &[Self::Pixel];
}

pub trait ImageWrite: ImageRead {
    fn data_mut(&mut self) -> &mut [Self::Pixel];
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

pub trait RectRange<T> {
    fn into_ranges(self, size: Size2D<T>) -> (Range<T>, Range<T>);
}

impl<X: RangeBounds<u32>, Y: RangeBounds<u32>> RectRange<u32> for (X, Y) {
    fn into_ranges(self, size: Size2D<u32>) -> (Range<u32>, Range<u32>) {
        (
            into_range(self.0, size.width),
            into_range(self.1, size.height),
        )
    }
}

impl RectRange<u32> for Rect<u32> {
    fn into_ranges(self, size: Size2D<u32>) -> (Range<u32>, Range<u32>) {
        (self.x_range(), self.y_range()).into_ranges(size)
    }
}

impl RectRange<u32> for Box2D<u32> {
    fn into_ranges(self, size: Size2D<u32>) -> (Range<u32>, Range<u32>) {
        (self.x_range(), self.y_range()).into_ranges(size)
    }
}

pub trait ImageReadExt: ImageRead {
    fn slice(&self, range: impl RectRange<u32>) -> ImageSlice<'_, Self::Pixel> {
        let size = self.size();
        let (x_range, y_range) = range.into_ranges(size);
        let data_start = x_range.start as usize + self.stride() as usize * y_range.start as usize;
        let data_end = if x_range.is_empty() || y_range.is_empty() {
            data_start
        } else {
            (x_range.end - 1) as usize + self.stride() as usize * (y_range.end - 1) as usize + 1
        };
        ImageSlice {
            size: Size2D::new(x_range.end - x_range.start, y_range.end - y_range.start),
            stride: self.stride(),
            data: &self.data()[data_start..data_end],
        }
    }

    fn get(&self, i: u32, j: u32) -> &Self::Pixel {
        let size = self.size();
        let stride = self.stride();
        assert!(
            i < size.width && j < size.height,
            "({i}, {j}) is out of bounds {size:?}"
        );
        &self.data()[i as usize + stride as usize * j as usize]
    }

    fn rows(&self) -> impl ExactSizeIterator<Item = (u32, &[Self::Pixel])> {
        let size = self.size();
        let stride = self.stride();
        self.data()
            .chunks(stride as usize)
            .enumerate()
            .map(move |(j, row)| (j as u32, row.split_at(size.width as usize).0))
    }

    fn pixels(&self) -> impl Iterator<Item = (Point2D<u32>, &Self::Pixel)> {
        self.rows().flat_map(|(j, row)| {
            row.iter()
                .enumerate()
                .map(move |(i, pixel)| (Point2D::new(i as u32, j), pixel))
        })
    }
}

pub trait ImageWriteMut: ImageWrite {
    fn slice_mut(&mut self, range: impl RectRange<u32>) -> ImageSliceMut<'_, Self::Pixel> {
        let size = self.size();
        let (x_range, y_range) = range.into_ranges(size);
        let data_start = x_range.start as usize + self.stride() as usize * y_range.start as usize;
        let data_end = if x_range.is_empty() || y_range.is_empty() {
            data_start
        } else {
            (x_range.end - 1) as usize + self.stride() as usize * (y_range.end - 1) as usize + 1
        };
        ImageSliceMut {
            size: Size2D::new(x_range.end - x_range.start, y_range.end - y_range.start),
            stride: self.stride(),
            data: &mut self.data_mut()[data_start..data_end],
        }
    }

    fn get_mut(&mut self, point: Point2D<u32>) -> &mut Self::Pixel {
        let size = self.size();
        let stride = self.stride();
        assert!(
            point.x < size.width && point.y < size.height,
            "{point:?} is out of bounds {size:?}"
        );
        &mut self.data_mut()[point.x as usize + stride as usize * point.y as usize]
    }

    fn rows_mut(&mut self) -> impl ExactSizeIterator<Item = (u32, &mut [Self::Pixel])> {
        let size = self.size();
        let stride = self.stride();
        self.data_mut()
            .chunks_mut(stride as usize)
            .enumerate()
            .map(move |(j, row)| (j as u32, row.split_at_mut(size.width as usize).0))
    }

    fn pixels_mut(&mut self) -> impl Iterator<Item = (Point2D<u32>, &mut Self::Pixel)> {
        self.rows_mut().flat_map(|(j, row)| {
            row.iter_mut()
                .enumerate()
                .map(move |(i, pixel)| (Point2D::new(i as u32, j), pixel))
        })
    }

    fn copy_from(&mut self, src: impl ImageRead<Pixel = Self::Pixel>) {
        assert_eq!(self.size(), src.size());
        for ((_, dst), (_, src)) in self.rows_mut().zip(src.rows()) {
            dst.copy_from_slice(src);
        }
    }

    fn copy_within(&mut self, src_rect: Rect<u32>, dst_origin: Point2D<u32>) {
        let all_rect = Rect::from_size(self.size());
        assert!(all_rect.contains_rect(&src_rect));
        if src_rect.origin == dst_origin {
            return;
        }
        let dst_rect = Rect {
            origin: dst_origin,
            size: src_rect.size,
        };
        assert!(all_rect.contains_rect(&dst_rect));

        let stride = self.stride() as usize;
        let origin_offset = |origin: Point2D<u32>| origin.x as usize + origin.y as usize * stride;
        let src_offset = origin_offset(src_rect.origin);
        let dst_offset = origin_offset(dst_rect.origin);

        let data = self.data_mut();
        let mut copy_line = |index: usize| {
            let line_offset = src_offset + index * stride;
            data.copy_within(
                line_offset..(line_offset + src_rect.size.width as usize),
                dst_offset + index * stride,
            );
        };

        if src_offset > dst_offset {
            for index in 0..(src_rect.size.height as usize) {
                copy_line(index);
            }
        } else {
            for index in (0..(src_rect.size.height as usize)).rev() {
                copy_line(index);
            }
        }
    }
}

impl<Q: ImageRead> ImageReadExt for Q {}
impl<Q: ImageWrite> ImageWriteMut for Q {}

pub trait ImageResize: ImageBase {
    /// Resize image copying old data.
    /// New pixels can contain arbitrary data.
    fn resize(&mut self, new_size: impl Into<Size2D<u32>>) {
        self.resize_with_fill(new_size, Self::Pixel::default());
    }

    /// Resize image copying old data.
    /// New pixels are filled with `fill` value.
    fn resize_with_fill(&mut self, new_size: impl Into<Size2D<u32>>, fill: Self::Pixel);
}

pub trait WithImage: ImageBase {
    fn with_image<F, R>(&self, f: F) -> R
    where
        F: FnOnce(ImageSlice<Self::Pixel>) -> R,
    {
        self.with_image_slice(f, Rect::from_size(self.size()))
    }

    fn with_image_slice<F, R>(&self, f: F, rect: Rect<u32>) -> R
    where
        F: FnOnce(ImageSlice<Self::Pixel>) -> R;
}

pub trait WithImageMut: ImageBase {
    fn with_image_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(ImageSliceMut<Self::Pixel>) -> R,
    {
        self.with_image_slice_mut(f, Rect::from_size(self.size()))
    }

    fn with_image_slice_mut<F, R>(&mut self, f: F, rect: Rect<u32>) -> R
    where
        F: FnOnce(ImageSliceMut<Self::Pixel>) -> R;
}

impl<Q: ImageRead> WithImage for Q {
    fn with_image_slice<F, R>(&self, f: F, rect: Rect<u32>) -> R
    where
        F: FnOnce(ImageSlice<Self::Pixel>) -> R,
    {
        f(self.slice((rect.x_range(), rect.y_range())))
    }
}

impl<Q: ImageWrite> WithImageMut for Q {
    fn with_image_slice_mut<F, R>(&mut self, f: F, rect: Rect<u32>) -> R
    where
        F: FnOnce(ImageSliceMut<Self::Pixel>) -> R,
    {
        f(self.slice_mut((rect.x_range(), rect.y_range())))
    }
}
