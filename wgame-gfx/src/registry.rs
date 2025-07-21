use crate::{Graphics, utils::AnyKey};
use alloc::{boxed::Box, rc::Rc};
use core::{any::Any, cell::RefCell, hash::Hash};
use hashbrown::{HashMap, hash_map::Entry};

#[derive(Default)]
struct AnyMap(HashMap<Box<dyn AnyKey>, Box<dyn Any>>);

impl AnyMap {
    pub fn insert<K, V>(&mut self, key: K, value: V) -> &mut V
    where
        K: Eq + Hash + 'static,
        V: Any + 'static,
    {
        match self.0.entry(Box::new(key) as Box<dyn AnyKey>) {
            Entry::Occupied(_) => panic!("Key already exists"),
            Entry::Vacant(entry) => entry.insert(Box::new(value) as Box<dyn Any>),
        }
        .downcast_mut::<V>()
        .expect("Registry value downcast error during insert")
    }

    pub fn get<K, V>(&self, key: &K) -> Option<&V>
    where
        K: Eq + Hash + 'static,
        V: Any + 'static,
    {
        Some(
            self.0
                .get(key as &dyn AnyKey)?
                .downcast_ref::<V>()
                .expect("Registry value downcast error during get"),
        )
    }
}

pub trait RegistryKey: Eq + Hash + 'static {
    type Value: Any + Clone + 'static;
}

pub trait RegistryInit: RegistryKey {
    fn create_value(&self, state: &Graphics) -> Self::Value;
}

/// Registry for shared resources
#[derive(Default)]
pub struct Registry {
    items: Rc<RefCell<AnyMap>>,
}

impl Registry {
    pub fn get_or_init<K: RegistryInit>(&self, key: K, state: &Graphics) -> K::Value {
        if let Some(value) = self.get(&key) {
            return value;
        }
        let value = key.create_value(state);
        self.items.borrow_mut().insert(key, value).clone()
    }

    pub fn get<K: RegistryKey>(&self, key: &K) -> Option<K::Value> {
        self.items.borrow().get(key).cloned()
    }
}
