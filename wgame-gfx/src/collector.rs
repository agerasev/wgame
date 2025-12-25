use std::{any::Any, rc::Rc};

use hashbrown::{HashMap, hash_map::EntryRef};

use crate::{AnyResource, AnyStorage, Context, Instance, Object};

pub trait InstanceVisitor<C: Context> {
    fn visit<T: Instance<Context = C>>(&mut self, instance: &T);
}

pub struct Collector<C: Context> {
    storages: HashMap<Rc<dyn AnyResource>, Box<dyn AnyStorage<C>>>,
}

impl<C: Context> Default for Collector<C> {
    fn default() -> Self {
        Self {
            storages: HashMap::default(),
        }
    }
}

impl<C: Context> Collector<C> {
    fn insert_instance<T: Instance<Context = C>>(&mut self, instance: &T) {
        let resource = instance.resource();
        let any_storage = match self.storages.entry_ref(&resource as &dyn AnyResource) {
            EntryRef::Vacant(entry) => entry.insert(Box::new(instance.new_storage())),
            EntryRef::Occupied(entry) => entry.into_mut(),
        };
        let storage = (any_storage.as_mut() as &mut dyn Any)
            .downcast_mut::<T::Storage>()
            .expect("Error downcasting storage during push");
        instance.store(storage);
    }

    pub fn insert<T: Object<Context = C>>(&mut self, object: &T) {
        object.for_each_instance(self);
    }

    pub fn is_empty(&self) -> bool {
        self.storages.is_empty()
    }
    pub fn len(&self) -> usize {
        self.storages.len()
    }
    pub fn iter(&self) -> impl Iterator<Item = &dyn AnyStorage<C>> {
        self.storages.values().map(|s| &**s)
    }
}

impl<C: Context> InstanceVisitor<C> for Collector<C> {
    fn visit<T: Instance<Context = C>>(&mut self, instance: &T) {
        self.insert_instance(instance);
    }
}
