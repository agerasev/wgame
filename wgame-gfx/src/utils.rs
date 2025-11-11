use std::{
    any::Any,
    cmp::Ordering,
    hash::{Hash, Hasher},
};

pub trait AnyKey: Any + 'static {
    fn hash_dyn(&self, state: &mut dyn Hasher);
    fn eq_dyn(&self, other: &dyn AnyKey) -> bool;
    fn cmp_dyn(&self, other: &dyn AnyKey) -> Ordering;
}

impl<T: Ord + Hash + 'static> AnyKey for T {
    fn hash_dyn(&self, mut state: &mut dyn Hasher) {
        self.hash(&mut state);
    }
    fn eq_dyn(&self, other: &dyn AnyKey) -> bool {
        if let Some(other) = (other as &dyn Any).downcast_ref::<T>() {
            self.eq(other)
        } else {
            false
        }
    }
    fn cmp_dyn(&self, other: &dyn AnyKey) -> Ordering {
        match self.type_id().cmp(&other.type_id()) {
            Ordering::Equal => self.cmp(
                (other as &dyn Any)
                    .downcast_ref::<T>()
                    .expect("Type IDs are equal, but downcast failed"),
            ),
            not_equal => not_equal,
        }
    }
}

impl PartialEq for dyn AnyKey {
    fn eq(&self, other: &Self) -> bool {
        self.eq_dyn(other)
    }
}

impl Eq for dyn AnyKey {}

impl Hash for dyn AnyKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_dyn(state);
    }
}

pub trait AnyOrder {
    fn order(&self) -> i64 {
        0
    }
}

impl<T: AnyOrder + ?Sized> AnyOrder for Box<T> {
    fn order(&self) -> i64 {
        (**self).order()
    }
}
