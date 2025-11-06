use alloc::rc::{Rc, Weak};
use core::cell::{Cell, RefCell};
use euclid::default::{Point2D, Rect, Size2D};
use guillotiere::{Allocation, AtlasAllocator};
use hashbrown::HashMap;
use smallvec::SmallVec;

use crate::{Image, ImageSlice, ImageSliceMut, Pixel, prelude::*};

#[derive(Default)]
pub struct Tracker {
    rect: Cell<Option<Rect<u32>>>,
}

impl Tracker {
    pub fn add(&self, rect: Rect<u32>) {
        self.rect.update(|old| {
            Some(match old {
                Some(old) => old.union(&rect),
                None => rect,
            })
        })
    }
    pub fn clear(&self) {
        self.rect.take();
    }
    pub fn take_next(&self) -> Option<Rect<u32>> {
        self.rect.take()
    }
}

struct InnerAtlas<P: Pixel> {
    allocator: AtlasAllocator,
    items: HashMap<ItemId, Allocation>,
    counter: ItemId,
    image: Image<P>,
    tracker: Weak<Tracker>,
}

#[derive(Clone)]
pub struct Atlas<P: Pixel> {
    inner: Rc<RefCell<InnerAtlas<P>>>,
}

type ItemId = u64;

struct AtlasItem<P: Pixel> {
    atlas: Rc<RefCell<InnerAtlas<P>>>,
    id: ItemId,
    size: Size2D<u32>,
}

#[derive(Clone)]
pub struct AtlasImage<P: Pixel> {
    inner: Rc<RefCell<AtlasItem<P>>>,
}

impl<P: Pixel> InnerAtlas<P> {
    fn alloc_item(&mut self, size: Size2D<u32>, id: Option<ItemId>) -> ItemId {
        let mut atlas_size = self.allocator.size();
        let mut items_to_alloc = SmallVec::<[(ItemId, Size2D<i32>); 1]>::new();
        let id = id.unwrap_or_else(|| {
            let id = self.counter;
            self.counter += 1;
            id
        });
        items_to_alloc.push((id, size.cast()));
        let mut init_items = None;

        loop {
            items_to_alloc.retain(|(id, size)| match self.allocator.allocate(*size) {
                Some(alloc) => {
                    assert!(self.items.insert(*id, alloc).is_none());
                    false
                }
                None => true,
            });
            if items_to_alloc.is_empty() {
                break;
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

            // TODO: Rearrange only if needed
            let change_list = self.allocator.resize_and_rearrange(atlas_size);
            let alloc_to_id = self
                .items
                .iter()
                .map(|(id, alloc)| (alloc.id, *id))
                .collect::<HashMap<_, _>>();
            assert_eq!(self.items.len(), alloc_to_id.len());
            if init_items.is_none() {
                init_items = Some(self.items.clone());
            }

            for failure in change_list.failures {
                let id = alloc_to_id[&failure.id];
                self.items.remove_entry(&id).unwrap();
                items_to_alloc.push((id, failure.rectangle.size()));
            }
            for change in change_list.changes {
                let id = alloc_to_id[&change.old.id];
                *self.items.get_mut(&id).unwrap() = change.new;
            }
        }

        if let Some(old_items) = init_items {
            let mut new_image = Image::new(atlas_size.cast());
            for (id, old_item) in old_items {
                new_image
                    .slice_mut(self.items[&id].rectangle.cast())
                    .copy_from(self.image.slice(old_item.rectangle.cast()));
            }
            self.image = new_image;

            if let Some(tracker) = self.tracker.upgrade() {
                tracker.clear();
                tracker.add(Rect::from_size(atlas_size.cast()));
            }
        }

        self.track_update(id, None);
        id
    }

    fn dealloc_item(&mut self, id: ItemId) -> Rect<u32> {
        let alloc = self.items.remove(&id).unwrap();
        self.allocator.deallocate(alloc.id);
        alloc.rectangle.to_rect().cast()
    }

    fn item_rect(&self, id: ItemId) -> Rect<u32> {
        self.items[&id].rectangle.to_rect().cast()
    }
    fn item_image(&self, id: ItemId) -> ImageSlice<'_, P> {
        self.image.slice(self.items[&id].rectangle.cast())
    }
    fn item_image_mut(&mut self, id: ItemId) -> ImageSliceMut<'_, P> {
        self.image.slice_mut(self.items[&id].rectangle.cast())
    }

    fn resize_item(&mut self, id: ItemId, new_size: Size2D<u32>) {
        let image = self.item_image(id).to_image();

        self.dealloc_item(id);
        self.alloc_item(new_size, Some(id));

        let common_size = new_size.min(image.size());
        self.item_image_mut(id)
            .slice_mut(Rect::from_size(common_size))
            .copy_from(image.slice(Rect::from_size(common_size)));
    }

    fn track_update(&mut self, id: ItemId, rect: Option<Rect<u32>>) {
        if let Some(tracker) = self.tracker.upgrade() {
            let item_rect = self.item_rect(id);
            let rect = match rect {
                Some(rect) => {
                    assert!(
                        rect.size.width <= item_rect.size.width
                            && rect.size.height <= item_rect.size.height
                    );
                    Rect {
                        origin: item_rect.origin + rect.origin.to_vector(),
                        size: rect.size,
                    }
                }
                None => item_rect,
            };
            tracker.add(rect);
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
                items: HashMap::new(),
                counter: 0,
                image: Image::new(size),
                tracker: Weak::default(),
            })),
        }
    }

    pub fn allocate(&self, size: impl Into<Size2D<u32>>) -> AtlasImage<P> {
        // TODO: Reserve 1px border
        let size = size.into();
        let id = self.inner.borrow_mut().alloc_item(size, None);
        let item = Rc::new(RefCell::new(AtlasItem {
            atlas: self.inner.clone(),
            id,
            size,
        }));
        AtlasImage { inner: item }
    }

    pub fn subscribe(&mut self, tracker: Weak<Tracker>) {
        let mut inner = self.inner.borrow_mut();
        assert!(
            inner.tracker.upgrade().is_none(),
            "Someone already subscribed"
        );
        inner.tracker = tracker;
    }
    pub fn unsubscribe(&mut self) {
        self.inner.borrow_mut().tracker = Weak::default();
    }

    pub fn size(&self) -> Size2D<u32> {
        self.inner.borrow().allocator.size().cast()
    }

    pub fn with_data<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Image<P>) -> R,
    {
        f(&self.inner.borrow().image)
    }
}

impl<P: Pixel> Drop for AtlasItem<P> {
    fn drop(&mut self) {
        self.atlas.borrow_mut().dealloc_item(self.id);
    }
}

impl<P: Pixel> AtlasItem<P> {
    pub fn rect(&self) -> Rect<u32> {
        self.atlas.borrow().item_rect(self.id)
    }

    fn resize(&mut self, new_size: Size2D<u32>) {
        self.atlas.borrow_mut().resize_item(self.id, new_size);
        self.size = new_size;
    }
}

impl<P: Pixel> AtlasImage<P> {
    pub fn rect(&self) -> Rect<u32> {
        self.inner.borrow().rect()
    }
    pub fn size(&self) -> Size2D<u32> {
        self.inner.borrow().size
    }

    pub fn atlas(&self) -> Atlas<P> {
        Atlas {
            inner: self.inner.borrow().atlas.clone(),
        }
    }

    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(ImageSlice<P>) -> R,
    {
        let this = self.inner.borrow();
        f(this.atlas.borrow().item_image(this.id))
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
        let mut atlas = this.atlas.borrow_mut();
        atlas.track_update(this.id, Some(rect));
        f(atlas.item_image_mut(this.id).slice_mut(rect))
    }

    pub fn resize(&mut self, new_size: impl Into<Size2D<u32>>) {
        // TODO: Reserve 1px border
        self.inner.borrow_mut().resize(new_size.into());
    }

    pub fn from_single(image: Image<P>) -> Self {
        let size = image.size();
        let mut allocator = AtlasAllocator::new(size.cast());
        let alloc = allocator.allocate(size.cast()).unwrap();
        assert_eq!(alloc.rectangle.min, Point2D::new(0, 0));
        let atlas = Rc::new(RefCell::new(InnerAtlas {
            allocator,
            image,
            items: [(0, alloc)].into_iter().collect(),
            counter: 1,
            tracker: Weak::default(),
        }));
        Self {
            inner: Rc::new(RefCell::new(AtlasItem { atlas, id: 0, size })),
        }
    }
}

impl<P: Pixel> PartialEq for Atlas<P> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}
impl<P: Pixel> Eq for Atlas<P> {}
