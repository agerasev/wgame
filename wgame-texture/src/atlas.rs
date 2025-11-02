use alloc::{
    collections::vec_deque::VecDeque,
    rc::{Rc, Weak},
    vec::Vec,
};
use core::cell::RefCell;
use euclid::default::{Rect, Size2D};
use guillotiere::{AllocId, Allocation, AtlasAllocator};
use wgame_img::{ImageSlice, ImageSliceMut, prelude::*};

pub trait ImageModifier: ImageResize + WithImage + WithImageMut {}
impl<Q: ImageModifier + WithImage + WithImageMut> ImageModifier for Q {}

pub type UpdateNotifier = Weak<RefCell<VecDeque<Rect<u32>>>>;
pub trait ImageWatcher: WithImage {
    fn subscribe_to_updates(&mut self, notifier: UpdateNotifier);
}

struct InnerAtlas<Q: ImageModifier> {
    allocator: AtlasAllocator,
    image: Q,
    updates: Vec<UpdateNotifier>,
}

#[derive(Clone)]
pub struct Atlas<Q: ImageModifier> {
    inner: Rc<RefCell<InnerAtlas<Q>>>,
}

struct AtlasItem<Q: ImageModifier> {
    atlas: Rc<RefCell<InnerAtlas<Q>>>,
    alloc: Allocation,
}

#[derive(Clone)]
pub struct AtlasImage<Q: ImageModifier> {
    inner: Rc<RefCell<AtlasItem<Q>>>,
}

impl<Q: ImageModifier> InnerAtlas<Q> {
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
        self.updates.retain_mut(|queue| match queue.upgrade() {
            Some(queue) => {
                let mut queue = queue.borrow_mut();
                queue.clear();
                queue.push_back(Rect::from_size(size.cast()));
                true
            }
            None => false,
        });
        alloc
    }

    fn alloc_item(&mut self, size: Size2D<i32>) -> Allocation {
        // TODO: Reserve 1px border
        self.allocate_growing(size)
    }

    fn dealloc_item(&mut self, id: AllocId) {
        self.allocator.deallocate(id);
    }

    fn notify_update(&mut self, rect: Rect<u32>) {
        self.updates.retain_mut(|queue| match queue.upgrade() {
            Some(queue) => {
                queue.borrow_mut().push_back(rect);
                true
            }
            None => false,
        });
    }
}

impl<Q: ImageModifier> Atlas<Q> {
    pub fn new(image: Q) -> Self {
        let size = image.size().cast::<i32>();
        Self {
            inner: Rc::new(RefCell::new(InnerAtlas {
                allocator: AtlasAllocator::new(size),
                image,
                updates: Vec::new(),
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

impl<Q: ImageModifier> ImageBase for Atlas<Q> {
    type Pixel = Q::Pixel;

    fn size(&self) -> Size2D<u32> {
        self.inner.borrow().image.size()
    }
}

impl<Q: ImageModifier> WithImage for Atlas<Q> {
    fn with_image_slice<F, R>(&self, f: F, rect: Rect<u32>) -> R
    where
        F: FnOnce(ImageSlice<Self::Pixel>) -> R,
    {
        self.inner.borrow().image.with_image_slice(f, rect)
    }
}

impl<Q: ImageModifier> ImageWatcher for Atlas<Q> {
    fn subscribe_to_updates(&mut self, notifier: UpdateNotifier) {
        self.inner.borrow_mut().updates.push(notifier);
    }
}

impl<Q: ImageModifier> Drop for AtlasItem<Q> {
    fn drop(&mut self) {
        self.atlas.borrow_mut().dealloc_item(self.alloc.id);
    }
}

impl<Q: ImageModifier> AtlasItem<Q> {
    fn resize(&mut self, new_size: Size2D<u32>) {
        let old_rect = self.alloc.rectangle.to_rect();
        let new_size = new_size.cast::<i32>();

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
}

impl<Q: ImageModifier> ImageBase for AtlasImage<Q> {
    type Pixel = Q::Pixel;

    fn size(&self) -> Size2D<u32> {
        let this = self.inner.borrow();
        this.alloc.rectangle.size().cast()
    }
}

impl<Q: ImageModifier> WithImage for AtlasImage<Q> {
    fn with_image_slice<F, R>(&self, f: F, rect: Rect<u32>) -> R
    where
        F: FnOnce(ImageSlice<Self::Pixel>) -> R,
    {
        let this = self.inner.borrow();
        this.atlas.borrow().image.with_image_slice(
            |image| f(image.slice(this.alloc.rectangle.cast::<u32>())),
            rect,
        )
    }
}

impl<Q: ImageModifier> WithImageMut for AtlasImage<Q> {
    fn with_image_slice_mut<F, R>(&mut self, f: F, rect: Rect<u32>) -> R
    where
        F: FnOnce(ImageSliceMut<Self::Pixel>) -> R,
    {
        let this = self.inner.borrow();
        let mut atlas = this.atlas.borrow_mut();
        atlas.notify_update(rect);
        atlas.image.with_image_slice_mut(
            |mut image| f(image.slice_mut(this.alloc.rectangle.cast::<u32>())),
            rect,
        )
    }
}

impl<Q: ImageModifier> ImageResize for AtlasImage<Q> {
    fn resize(&mut self, new_size: impl Into<Size2D<u32>>) {
        self.inner.borrow_mut().resize(new_size.into());
    }
    fn resize_with_fill(&mut self, _new_size: impl Into<Size2D<u32>>, _fill: Self::Pixel) {
        unimplemented!()
    }
}
