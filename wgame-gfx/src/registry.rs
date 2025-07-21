use alloc::{boxed::Box, rc::Rc};
use core::{any::Any, cell::RefCell, hash::Hash};

use hashbrown::HashMap;

use crate::utils::AnyKey;

#[derive(Default)]
struct AnyMap(HashMap<Box<dyn AnyKey>, Box<dyn Any>>);

impl AnyMap {
    pub fn get_or_init<K, V, F>(&mut self, key: K, init: F) -> &mut V
    where
        K: Eq + Hash + 'static,
        V: Any + 'static,
        F: FnOnce(&K) -> V,
    {
        self.0
            .entry(Box::new(key) as Box<dyn AnyKey>)
            .or_insert_with_key(|k| {
                Box::new(init(
                    (&**k as &dyn Any)
                        .downcast_ref::<K>()
                        .expect("Registry key downcast error"),
                )) as Box<dyn Any>
            })
            .downcast_mut::<V>()
            .expect("Registry value downcast error during get_or_init")
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
    fn create_value(&self, device: &wgpu::Device) -> Self::Value;
}

/// Registry for shared resources
pub struct Registry {
    items: Rc<RefCell<AnyMap>>,
    device: wgpu::Device,
}

impl Registry {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            items: Default::default(),
            device: device.clone(),
        }
    }

    pub fn get_or_init<K: RegistryInit>(&self, key: K) -> K::Value {
        self.items
            .borrow_mut()
            .get_or_init(key, |k| k.create_value(&self.device))
            .clone()
    }

    pub fn get<K: RegistryKey>(&self, key: &K) -> Option<K::Value> {
        self.items.borrow().get(key).cloned()
    }
}
