use std::{any::Any, rc::Rc};

use hashbrown::{HashMap, hash_map::EntryRef};

use crate::{AnyResource, AnyStorage, Context, Instance};

pub trait InstanceVisitor<C: Context> {
    fn visit<T: Instance<Context = C>>(&mut self, instance: &T);
}

#[derive(Default)]
pub struct Collector<C: Context> {
    storages: HashMap<Rc<dyn AnyResource>, Box<dyn AnyStorage<C>>>,
}

impl<C: Context> Collector<C> {
    fn insert<T: Instance<Context = C>>(&mut self, instance: &T) {
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

    pub fn storages(&self) -> impl Iterator<Item = &dyn AnyStorage<C>> {
        self.storages.values().map(|s| &**s)
    }
}

impl<C: Context> InstanceVisitor<C> for Collector<C> {
    fn visit<T: Instance<Context = C>>(&mut self, instance: &T) {
        self.insert(instance);
    }
}
