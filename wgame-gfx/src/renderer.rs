use std::{any::Any, rc::Rc};

use hashbrown::{HashMap, hash_map::EntryRef};

use crate::{Camera, Instance, Resource, resource::AnyResource};

pub trait InstanceVisitor {
    fn visit<T: Instance>(&mut self, instance: T);
}

#[derive(Default)]
pub struct Collector {
    items: HashMap<Rc<dyn AnyResource>, Box<dyn Any>>,
}

impl Collector {
    fn insert<T: Instance>(&mut self, instance: T) {
        let resource = instance.resource();
        let storage = self.get_or_init_storage(resource);
        instance.store(storage);
    }

    fn get_or_init_storage<R: Resource>(&mut self, resource: R) -> &mut R::Storage {
        match self.items.entry_ref(resource.as_any()) {
            EntryRef::Vacant(entry) => entry.insert(resource.as_any().new_dyn_storage()),
            EntryRef::Occupied(entry) => entry.into_mut(),
        }
        .downcast_mut()
        .expect("Error downcasting storage during push")
    }

    pub fn items(&self) -> impl Iterator<Item = (&dyn AnyResource, &dyn Any)> {
        self.items.iter().map(|(r, s)| (&**r, &**s))
    }
}

impl InstanceVisitor for Collector {
    fn visit<T: Instance>(&mut self, instance: T) {
        self.insert(instance);
    }
}

pub struct Renderer {
    pub collector: Collector,
    pub camera: Camera,
}

impl InstanceVisitor for Renderer {
    fn visit<T: Instance>(&mut self, instance: T) {
        self.collector.insert(instance);
    }
}
