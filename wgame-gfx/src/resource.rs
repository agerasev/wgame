use std::{
    any::Any,
    cmp::Ordering,
    hash::{Hash, Hasher},
    iter,
    rc::Rc,
};

use smallvec::SmallVec;

/// Shared resource required to draw an instance.
///
/// Equality of the instances' resource means that they can be draw in single render pass.
pub trait Resource: Any + Eq + Ord + Hash + Clone + Sized {
    fn order(&self) -> impl Iterator<Item = i32> {
        iter::empty()
    }
}

pub trait AnyResource: Any + 'static {
    fn clone_dyn(&self) -> Rc<dyn AnyResource>;
    fn hash_dyn(&self, state: &mut dyn Hasher);
    fn eq_dyn(&self, other: &dyn AnyResource) -> bool;
    fn cmp_dyn(&self, other: &dyn AnyResource) -> Ordering;
    fn order_dyn(&self) -> SmallVec<[i32; 4]>;
}

impl<R: Resource> AnyResource for R {
    fn clone_dyn(&self) -> Rc<dyn AnyResource> {
        Rc::new(self.clone())
    }

    fn hash_dyn(&self, mut state: &mut dyn Hasher) {
        self.hash(&mut state);
    }

    fn eq_dyn(&self, other: &dyn AnyResource) -> bool {
        if let Some(other) = (other as &dyn Any).downcast_ref::<R>() {
            self.eq(other)
        } else {
            false
        }
    }

    fn cmp_dyn(&self, other: &dyn AnyResource) -> Ordering {
        match self.type_id().cmp(&other.type_id()) {
            Ordering::Equal => self.cmp(
                (other as &dyn Any)
                    .downcast_ref::<R>()
                    .expect("Type IDs are equal, but downcast failed"),
            ),
            not_equal => not_equal,
        }
    }

    fn order_dyn(&self) -> SmallVec<[i32; 4]> {
        self.order().collect()
    }
}

impl PartialEq for dyn AnyResource {
    fn eq(&self, other: &dyn AnyResource) -> bool {
        self.eq_dyn(other)
    }
}
impl Eq for dyn AnyResource {}
impl PartialOrd for dyn AnyResource {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for dyn AnyResource {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_dyn(other)
    }
}
impl Hash for dyn AnyResource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_dyn(state);
    }
}

impl Resource for Rc<dyn AnyResource> {
    fn order(&self) -> impl Iterator<Item = i32> {
        (**self).order_dyn().into_iter()
    }
}

impl From<&dyn AnyResource> for Rc<dyn AnyResource> {
    fn from(value: &dyn AnyResource) -> Self {
        value.clone_dyn()
    }
}
