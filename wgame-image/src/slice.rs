use euclid::default::Size2D;

use crate::{ImageBase, ImageRead, ImageWrite, Pixel};

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
