use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use euclid::default::{Point2D, Rect, Size2D};
use guillotiere::{Allocation, AtlasAllocator};
use hashbrown::HashMap;

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
    generation: u64,
    image: Image<P>,
    tracker: Weak<Tracker>,
}

/// Shared image atlas with append-only allocation generations.
/// Dropped/resized rectangles stay occupied until live items are repacked. On
/// exhaustion, at most 50% live area (including the pending allocation) permits
/// a same-size compaction attempt before growth. Handles follow relocation.
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
        assert!(
            size.width > 0
                && size.height > 0
                && size.width <= i32::MAX as u32
                && size.height <= i32::MAX as u32,
            "Image size must fit positive i32 dimensions"
        );
        let id = id.unwrap_or_else(|| {
            let id = self.counter;
            self.counter = self.counter.checked_add(1).expect("Atlas item ID overflow");
            id
        });
        if let Some(alloc) = self.allocator.allocate(size.cast()) {
            self.items.insert(id, alloc);
        } else {
            self.repack(id, size.cast());
        }
        self.track_update(id, None);
        id
    }

    fn repack(&mut self, id: ItemId, size: Size2D<i32>) {
        // Dead and resized allocations remain occupied in the current generation.
        // Only live items, plus the requested replacement, enter the next one.
        let mut requests: Vec<_> = self
            .items
            .iter()
            .filter(|(item_id, _)| **item_id != id)
            .map(|(id, alloc)| (*id, alloc.rectangle.size()))
            .chain([(id, size)])
            .collect();
        let area = |size: Size2D<i32>| size.width as u64 * size.height as u64;
        let live_area: u64 = requests.iter().map(|(_, size)| area(*size)).sum();
        requests.sort_unstable_by_key(|(id, size)| (std::cmp::Reverse(area(*size)), *id));
        let mut atlas_size = self.allocator.size();
        let grow = |size: &mut Size2D<i32>| {
            let side = if size.height < size.width {
                &mut size.height
            } else {
                &mut size.width
            };
            *side = side.checked_mul(2).expect("Atlas size overflow");
        };
        // Area includes the pending item and any padding supplied by the caller.
        // Even below this threshold, fragmentation or dimensions can require growth.
        if live_area > area(atlas_size) / 2 {
            grow(&mut atlas_size);
        }
        let (allocator, items) = loop {
            let mut allocator = AtlasAllocator::new(atlas_size);
            let items: Option<HashMap<_, _>> = requests
                .iter()
                .map(|(id, size)| allocator.allocate(*size).map(|alloc| (*id, alloc)))
                .collect();
            if let Some(items) = items {
                break (allocator, items);
            }
            grow(&mut atlas_size);
        };
        let mut image = Image::new(atlas_size.cast());
        for (old_id, old_alloc) in &self.items {
            if *old_id != id {
                image
                    .slice_mut(items[old_id].rectangle.cast())
                    .copy_from(self.image.slice(old_alloc.rectangle.cast()));
            }
        }
        self.allocator = allocator;
        self.items = items;
        self.image = image;
        self.generation = self
            .generation
            .checked_add(1)
            .expect("Atlas generation overflow");
        if let Some(tracker) = self.tracker.upgrade() {
            tracker.clear();
            tracker.add(Rect::from_size(atlas_size.cast()));
        }
    }

    fn remove_item(&mut self, id: ItemId) {
        // Do not return this rectangle to the allocator or clear its pixels:
        // baked/encoded draws may still reference it in this GPU generation.
        self.items.remove(&id).unwrap();
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

        // Allocate before replacing the live mapping; the old rectangle is never reused.
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
                        rect.origin.x <= item_rect.size.width
                            && rect.origin.y <= item_rect.size.height
                            && rect.size.width <= item_rect.size.width - rect.origin.x
                            && rect.size.height <= item_rect.size.height - rect.origin.y
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
        assert!(
            size.width > 0
                && size.height > 0
                && size.width <= i32::MAX as u32
                && size.height <= i32::MAX as u32,
            "Atlas size must fit positive i32 dimensions"
        );
        Self {
            inner: Rc::new(RefCell::new(InnerAtlas {
                allocator: AtlasAllocator::new(size.cast()),
                items: HashMap::new(),
                counter: 0,
                generation: 0,
                image: Image::new(size),
                tracker: Weak::default(),
            })),
        }
    }

    pub fn allocate(&self, size: impl Into<Size2D<u32>>) -> AtlasImage<P> {
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

    /// Changes whenever live items are repacked into a replacement atlas, even
    /// when its dimensions stay the same. GPU mirrors must replace their texture.
    pub fn generation(&self) -> u64 {
        self.inner.borrow().generation
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
        self.atlas.borrow_mut().remove_item(self.id);
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

    pub fn update<F, R>(&self, f: F) -> R
    where
        F: FnOnce(ImageSliceMut<P>) -> R,
    {
        self.update_part(f, Rect::from_size(self.size()))
    }

    pub fn update_part<F, R>(&self, f: F, rect: Rect<u32>) -> R
    where
        F: FnOnce(ImageSliceMut<P>) -> R,
    {
        let this = self.inner.borrow();
        let mut atlas = this.atlas.borrow_mut();
        atlas.track_update(this.id, Some(rect));
        f(atlas.item_image_mut(this.id).slice_mut(rect))
    }

    /// Allocate a replacement rectangle and preserve overlapping pixels.
    /// Cloned handles follow the new rectangle; the old one remains untouched
    /// within its generation for previously baked or encoded drawing.
    pub fn resize(&self, new_size: impl Into<Size2D<u32>>) {
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
            generation: 0,
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
