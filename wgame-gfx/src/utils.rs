use core::{
    any::Any,
    hash::{Hash, Hasher},
};

pub trait AnyKey: Any + 'static {
    fn hash_dyn(&self, state: &mut dyn Hasher);
    fn eq_dyn(&self, other: &dyn AnyKey) -> bool;
}

impl<T: Eq + Hash + 'static> AnyKey for T {
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
