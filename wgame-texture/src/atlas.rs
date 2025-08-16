use alloc::{
    collections::btree_map::BTreeMap,
    rc::{Rc, Weak},
};
use core::cell::RefCell;

use guillotiere::{AllocId, AtlasAllocator};

use crate::Texture;

struct AtlasItem {
    texture: Weak<Texture>,
}

pub(crate) struct InnerAtlas {
    allocator: AtlasAllocator,
    items: BTreeMap<AllocId, AtlasItem>,
}

pub struct Atlas {
    inner: Rc<RefCell<InnerAtlas>>,
}
