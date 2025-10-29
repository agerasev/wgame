use alloc::rc::Rc;
use core::cell::RefCell;
use euclid::default::{Rect, Size2D};
use guillotiere::{AllocId, Allocation, AtlasAllocator};
use wgame_img::{ImageSlice, ImageSliceMut, prelude::*};

pub trait ImageOrigin: ImageResize + WithImage + WithImageMut {}
impl<Q: ImageOrigin + WithImage + WithImageMut> ImageOrigin for Q {}

pub struct AtlasItem<Q: ImageOrigin> {
    atlas: Rc<RefCell<InnerAtlas<Q>>>,
    alloc: Allocation,
}

struct InnerAtlas<Q: ImageOrigin> {
    allocator: AtlasAllocator,
    image: Q,
}

#[derive(Clone)]
pub struct Atlas<Q: ImageOrigin> {
    inner: Rc<RefCell<InnerAtlas<Q>>>,
}

#[derive(Clone)]
struct AtlasImage<Q: ImageOrigin> {
    inner: Rc<RefCell<AtlasItem<Q>>>,
}

impl<Q: ImageOrigin> InnerAtlas<Q> {
    fn allocate_growing(&mut self, size: Size2D<i32>) -> Allocation {
        let mut atlas_size = self.allocator.size();
        let size = size.into();
        let alloc = loop {
            if let Some(alloc) = self.allocator.allocate(size) {
                break alloc;
            }
            if atlas_size.height < atlas_size.width {
                atlas_size.height = atlas_size
                    .height
                    .checked_mul(2)
                    .expect("Atlas size overflow");
            } else {
                atlas_size.width = atlas_size
                    .width
                    .checked_mul(2)
                    .expect("Atlas size overflow");
            }
            self.allocator.grow(atlas_size);
        };
        self.image.resize(size.cast());
        alloc
    }

    fn alloc_item(&mut self, size: Size2D<i32>) -> Allocation {
        // TODO: Reserve 1px border
        self.allocate_growing(size)
    }

    fn dealloc_item(&mut self, id: AllocId) {
        self.allocator.deallocate(id);
    }
}

impl<Q: ImageOrigin> Atlas<Q> {
    pub fn new(image: Q) -> Self {
        let size = image.size().cast::<i32>();
        Self {
            inner: Rc::new(RefCell::new(InnerAtlas {
                allocator: AtlasAllocator::new(size),
                image,
            })),
        }
    }

    pub fn allocate(&self, size: impl Into<Size2D<u32>>) -> AtlasImage<Q> {
        let size = size.into().cast::<i32>();
        let mut inner = self.inner.borrow_mut();
        let inner_item = AtlasItem {
            atlas: self.inner.clone(),
            alloc: inner.alloc_item(size),
        };
        let item = Rc::new(RefCell::new(inner_item));
        AtlasImage { inner: item }
    }
}

impl<Q: ImageOrigin> Drop for AtlasItem<Q> {
    fn drop(&mut self) {
        self.atlas.borrow_mut().dealloc_item(self.alloc.id);
    }
}

impl<Q: ImageOrigin> ImageBase for AtlasItem<Q> {
    type Pixel = Q::Pixel;

    fn size(&self) -> Size2D<u32> {
        self.alloc.rectangle.size().cast()
    }
}

impl<Q: ImageOrigin> WithImage for AtlasItem<Q> {
    fn with_image<F: FnOnce(ImageSlice<Self::Pixel>) -> R, R>(&self, f: F) -> R {
        self.atlas
            .borrow()
            .image
            .with_image(|image| f(image.slice(self.alloc.rectangle.cast::<u32>())))
    }
}

impl<Q: ImageOrigin> WithImageMut for AtlasItem<Q> {
    fn with_image_mut<F: FnOnce(ImageSliceMut<Self::Pixel>) -> R, R>(&mut self, f: F) -> R {
        self.atlas
            .borrow_mut()
            .image
            .with_image_mut(|mut image| f(image.slice_mut(self.alloc.rectangle.cast::<u32>())))
    }
}

impl<Q: ImageOrigin> ImageResize for AtlasItem<Q> {
    fn resize(&mut self, new_size: impl Into<Size2D<u32>>) {
        let old_rect = self.alloc.rectangle.to_rect();
        let new_size = new_size.into().cast::<i32>();

        let mut atlas = self.atlas.borrow_mut();

        atlas.dealloc_item(self.alloc.id);
        let new_alloc = atlas.alloc_item(new_size);

        let common_size = new_size.min(old_rect.size);
        atlas.image.with_image_mut(|mut image| {
            image.copy_within(
                Rect {
                    origin: old_rect.origin,
                    size: common_size,
                }
                .cast(),
                new_alloc.rectangle.min.cast(),
            )
        });
    }
    fn resize_with_fill(&mut self, _new_size: impl Into<Size2D<u32>>, _fill: Self::Pixel) {
        unimplemented!()
    }
}
