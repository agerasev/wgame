use std::ops::{Bound, Range, RangeBounds};

use euclid::default::{Box2D, Point2D, Rect, Size2D};

use crate::{Image, ImageSlice, ImageSliceMut, Pixel};

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

impl<Q: ImageBase + ?Sized> ImageBase for &Q {
    type Pixel = Q::Pixel;
    fn size(&self) -> Size2D<u32> {
        Q::size(self)
    }
}
impl<Q: ImageRead + ?Sized> ImageRead for &Q {
    fn data(&self) -> &[Self::Pixel] {
        Q::data(self)
    }
    fn stride(&self) -> u32 {
        Q::stride(self)
    }
}

impl<Q: ImageBase + ?Sized> ImageBase for &mut Q {
    type Pixel = Q::Pixel;
    fn size(&self) -> Size2D<u32> {
        Q::size(self)
    }
}
impl<Q: ImageRead + ?Sized> ImageRead for &mut Q {
    fn data(&self) -> &[Self::Pixel] {
        Q::data(self)
    }
    fn stride(&self) -> u32 {
        Q::stride(self)
    }
}
impl<Q: ImageWrite + ?Sized> ImageWrite for &mut Q {
    fn data_mut(&mut self) -> &mut [Self::Pixel] {
        Q::data_mut(self)
    }
}

fn into_range(range: impl RangeBounds<u32>, size: u32) -> Range<u32> {
    let start = match range.start_bound() {
        Bound::Included(x) => *x,
        Bound::Excluded(x) => x.checked_add(1).expect("Image range start overflows u32"),
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(x) => x.checked_add(1).expect("Image range end overflows u32"),
        Bound::Excluded(x) => *x,
        Bound::Unbounded => size,
    };
    assert!(start <= size, "Image range start is out of bounds");
    assert!(end <= size, "Image range end is out of bounds");
    assert!(start <= end, "Image range start exceeds its end");
    start..end
}

/// A rectangular region expressed as two coordinate ranges.
///
/// Range pairs accept all standard included, excluded, and unbounded bounds.
/// Empty ranges are valid, including at the image's right and bottom edges.
/// Rectangles and boxes use an inclusive origin and exclusive end.
pub trait RectRange<T> {
    /// Returns ranges within `size`, with each start no greater than its end.
    ///
    /// # Panics
    ///
    /// The provided implementations panic if a bound is outside `size`, a range
    /// is reversed, or normalizing a bound overflows.
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
        let end_x = self
            .origin
            .x
            .checked_add(self.size.width)
            .expect("Image rectangle x bound overflows u32");
        let end_y = self
            .origin
            .y
            .checked_add(self.size.height)
            .expect("Image rectangle y bound overflows u32");
        (self.origin.x..end_x, self.origin.y..end_y).into_ranges(size)
    }
}

impl RectRange<u32> for Box2D<u32> {
    fn into_ranges(self, size: Size2D<u32>) -> (Range<u32>, Range<u32>) {
        (self.x_range(), self.y_range()).into_ranges(size)
    }
}

fn slice_data_range(x: Range<u32>, y: Range<u32>, stride: u32) -> Range<usize> {
    // An empty region at an image edge can have an origin beyond its storage,
    // especially when slicing a view whose last row omits trailing padding.
    if x.is_empty() || y.is_empty() {
        0..0
    } else {
        (x.start as usize + stride as usize * y.start as usize)
            ..((x.end - 1) as usize + stride as usize * (y.end - 1) as usize + 1)
    }
}

pub trait ImageReadExt: ImageRead {
    /// Borrows a rectangular region, retaining the source row stride.
    ///
    /// An empty region has empty data and yields no rows or pixels.
    ///
    /// ```
    /// use std::ops::Bound::{Excluded, Included};
    /// use wgame_image::{Image, prelude::*};
    ///
    /// let image = Image::<u8>::with_data((3, 2), [0, 1, 2, 3, 4, 5]);
    /// assert_eq!(image.slice(((Excluded(0), Included(2)), 1..)).data(), [4, 5]);
    /// assert!(image.slice((3..3, 2..2)).data().is_empty());
    /// ```
    ///
    /// # Panics
    ///
    /// Panics for invalid bounds as described by [`RectRange::into_ranges`].
    fn slice(&self, range: impl RectRange<u32>) -> ImageSlice<'_, Self::Pixel> {
        let size = self.size();
        let (x_range, y_range) = range.into_ranges(size);
        ImageSlice {
            size: Size2D::new(x_range.end - x_range.start, y_range.end - y_range.start),
            stride: self.stride(),
            data: &self.data()[slice_data_range(x_range, y_range, self.stride())],
        }
    }

    fn get(&self, point: Point2D<u32>) -> &Self::Pixel {
        let size = self.size();
        let stride = self.stride();
        assert!(
            point.x < size.width && point.y < size.height,
            "{point:?} is out of bounds {size:?}"
        );
        &self.data()[point.x as usize + stride as usize * point.y as usize]
    }

    fn rows(&self) -> impl ExactSizeIterator<Item = (u32, &[Self::Pixel])> {
        let size = self.size();
        let stride = self.stride();
        self.data()
            .chunks((stride as usize).max(1))
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

    fn to_image(&self) -> Image<Self::Pixel> {
        let mut image = Image::new(self.size());
        image.copy_from(self);
        image
    }
}

pub trait ImageWriteMut: ImageWrite {
    /// Mutably borrows a region with the same bounds and empty-region behavior
    /// as [`ImageReadExt::slice`].
    ///
    /// # Panics
    ///
    /// Panics for invalid bounds as described by [`RectRange::into_ranges`].
    fn slice_mut(&mut self, range: impl RectRange<u32>) -> ImageSliceMut<'_, Self::Pixel> {
        let size = self.size();
        let (x_range, y_range) = range.into_ranges(size);
        let data_range = slice_data_range(x_range.clone(), y_range.clone(), self.stride());
        ImageSliceMut {
            size: Size2D::new(x_range.end - x_range.start, y_range.end - y_range.start),
            stride: self.stride(),
            data: &mut self.data_mut()[data_range],
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
            .chunks_mut((stride as usize).max(1))
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

    fn fill(&mut self, color: Self::Pixel) {
        for (_, row) in self.rows_mut() {
            row.fill(color);
        }
    }
}

impl<Q: ImageRead + ?Sized> ImageReadExt for Q {}
impl<Q: ImageWrite + ?Sized> ImageWriteMut for Q {}
