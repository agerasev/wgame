use euclid::default::{Rect, Size2D};

use crate::{ImageBase, ImageRead, ImageReadExt, ImageWrite, ImageWriteMut, Pixel};

pub struct Image<P: Pixel> {
    size: Size2D<u32>,
    data: Vec<P>,
}

impl<P: Pixel> Image<P> {
    pub fn new(size: impl Into<Size2D<u32>>) -> Self {
        Self::with_color(size, P::default())
    }

    pub fn with_color(size: impl Into<Size2D<u32>>, fill: P) -> Self {
        let size = size.into();
        Self::with_data(size, vec![fill; size.cast::<usize>().area()])
    }

    pub fn with_data(size: impl Into<Size2D<u32>>, data: impl Into<Vec<P>>) -> Self {
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

    pub fn resize(&mut self, new_size: impl Into<Size2D<u32>>, fill: P) {
        let new_size = new_size.into();
        if self.size == new_size {
            return;
        }
        let mut new_image = Image::with_color(new_size, fill);
        let common_rect = Rect::from_size(new_size.min(self.size));
        new_image
            .slice_mut(common_rect)
            .copy_from(self.slice(common_rect));
        *self = new_image
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
