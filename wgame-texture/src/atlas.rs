use alloc::{
    collections::vec_deque::VecDeque,
    rc::{Rc, Weak},
    vec,
};
use core::cell::RefCell;
use euclid::default::{Point2D, Rect, Size2D};
use guillotiere::{AllocId, Allocation, AtlasAllocator};
use wgame_img::{Image, ImageSlice, ImageSliceMut, Pixel, prelude::*};

pub struct Notifier {
    pub updates: RefCell<VecDeque<Rect<u32>>>,
}

struct InnerAtlas<P: Pixel> {
    allocator: AtlasAllocator,
    image: Image<P>,
    notifier: Weak<Notifier>,
}

#[derive(Clone)]
pub struct Atlas<P: Pixel> {
    inner: Rc<RefCell<InnerAtlas<P>>>,
}

struct AtlasItem<P: Pixel> {
    atlas: Rc<RefCell<InnerAtlas<P>>>,
    alloc: Allocation,
}

#[derive(Clone)]
pub struct AtlasImage<P: Pixel> {
    inner: Rc<RefCell<AtlasItem<P>>>,
}

impl<P: Pixel> InnerAtlas<P> {
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
        if let Some(notifier) = self.notifier.upgrade() {
            let mut queue = notifier.updates.borrow_mut();
            queue.clear();
            queue.push_back(Rect::from_size(size.cast()));
        }
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
        if let Some(notifier) = self.notifier.upgrade() {
            notifier.updates.borrow_mut().push_back(rect);
        }
    }
}

impl<P: Pixel> Default for Atlas<P> {
    fn default() -> Self {
        Self::with_size(Self::INITIAL_SIZE)
    }
}

impl<P: Pixel> Atlas<P> {
    const INITIAL_SIZE: Size2D<u32> = Size2D::new(16, 16);

    pub fn with_size(size: Size2D<u32>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(InnerAtlas {
                allocator: AtlasAllocator::new(size.cast()),
                image: Image::new(size, vec![P::default(); size.cast::<usize>().area()]),
                notifier: Weak::default(),
            })),
        }
    }

    pub fn allocate(&self, size: impl Into<Size2D<u32>>) -> AtlasImage<P> {
        let size = size.into().cast::<i32>();
        let mut inner = self.inner.borrow_mut();
        let inner_item = AtlasItem {
            atlas: self.inner.clone(),
            alloc: inner.alloc_item(size),
        };
        let item = Rc::new(RefCell::new(inner_item));
        AtlasImage { inner: item }
    }

    pub(crate) fn subscribe(&mut self, notifier: Weak<Notifier>) {
        let mut inner = self.inner.borrow_mut();
        assert!(
            inner.notifier.upgrade().is_none(),
            "Someone already subscribed"
        );
        inner.notifier = notifier;
    }
    pub(crate) fn unsubscribe(&mut self) {
        self.inner.borrow_mut().notifier = Weak::default();
    }

    pub fn size(&self) -> Size2D<u32> {
        self.inner.borrow().allocator.size().cast()
    }

    pub(crate) fn with_data<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Image<P>) -> R,
    {
        f(&self.inner.borrow().image)
    }
}

impl<P: Pixel> Drop for AtlasItem<P> {
    fn drop(&mut self) {
        self.atlas.borrow_mut().dealloc_item(self.alloc.id);
    }
}

impl<P: Pixel> AtlasItem<P> {
    fn rect(&self) -> Rect<u32> {
        self.alloc.rectangle.to_rect().cast()
    }

    fn resize(&mut self, new_size: Size2D<u32>) {
        let old_rect = self.alloc.rectangle.to_rect();
        let new_size = new_size.cast::<i32>();

        let mut atlas = self.atlas.borrow_mut();

        atlas.dealloc_item(self.alloc.id);
        let new_alloc = atlas.alloc_item(new_size);

        let common_size = new_size.min(old_rect.size);
        atlas.image.copy_within(
            Rect {
                origin: old_rect.origin,
                size: common_size,
            }
            .cast(),
            new_alloc.rectangle.min.cast(),
        );
    }
}

impl<P: Pixel> AtlasImage<P> {
    pub(crate) fn rect(&self) -> Rect<u32> {
        self.inner.borrow().rect()
    }
    pub fn size(&self) -> Size2D<u32> {
        self.rect().size
    }

    pub fn atlas(&self) -> Atlas<P> {
        Atlas {
            inner: self.inner.borrow().atlas.clone(),
        }
    }

    pub fn with_data<F, R>(&self, f: F) -> R
    where
        F: FnOnce(ImageSlice<P>) -> R,
    {
        let this = self.inner.borrow();
        f(this.atlas.borrow().image.slice(this.rect()))
    }

    pub fn update<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(ImageSliceMut<P>) -> R,
    {
        self.update_part(f, Rect::from_size(self.size()))
    }

    pub fn update_part<F, R>(&mut self, f: F, rect: Rect<u32>) -> R
    where
        F: FnOnce(ImageSliceMut<P>) -> R,
    {
        let this = self.inner.borrow();
        let this_rect = this.rect();
        assert!(
            rect.size.width <= this_rect.size.width && rect.size.height <= this_rect.size.height
        );
        let part_rect = Rect {
            origin: this_rect.origin + rect.origin.to_vector(),
            size: rect.size,
        };
        let mut atlas = this.atlas.borrow_mut();
        atlas.notify_update(part_rect);
        f(atlas.image.slice_mut(part_rect))
    }

    pub fn resize(&mut self, new_size: impl Into<Size2D<u32>>) {
        self.inner.borrow_mut().resize(new_size.into());
    }

    pub fn from_single(image: Image<P>) -> Self {
        let size = image.size().cast();
        let mut allocator = AtlasAllocator::new(size);
        let alloc = allocator.allocate(size).unwrap();
        assert_eq!(alloc.rectangle.min, Point2D::new(0, 0));
        let atlas = Rc::new(RefCell::new(InnerAtlas {
            allocator,
            image,
            notifier: Weak::default(),
        }));
        Self {
            inner: Rc::new(RefCell::new(AtlasItem { atlas, alloc })),
        }
    }
}
