use std::{any::Any, rc::Rc};

use hashbrown::{HashMap, hash_map::EntryRef};

use crate::{Camera, Context, Instance, Object, Resource, object::Visitor, resource::AnyResource};

#[derive(Default)]
pub struct Collector {
    items: HashMap<Rc<dyn AnyResource>, Box<dyn Any>>,
}

impl Collector {
    pub fn insert<T: Instance>(&mut self, params: &T::Context, instance: T) {
        let resource = instance.resource();
        let instances = self.get_or_init_storage(resource);
        instance.store(params, instances);
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
        let mut items: Vec<_> = self.items.iter().collect();
        items.sort_by_key(|(r, _)| (r.order(), &***r));
        items.into_iter().map(|(r, s)| (&**r, &**s))
    }
}

impl<C: Context> Visitor<C> for Collector {
    fn add<T: Instance<Context = C>>(&mut self, instance: T) {
        self.insert(&C::default(), instance);
    }
}

pub struct CollectorWithContext<'a, C: Context = Camera> {
    pub collector: &'a mut Collector,
    pub context: C,
}

impl<'a, C: Context> CollectorWithContext<'a, C> {
    pub fn add<T: Object<Context = C>>(&mut self, object: T) {
        object.draw(self);
    }
}

impl<C: Context> Visitor<C> for CollectorWithContext<'_, C> {
    fn add<T: Instance<Context = C>>(&mut self, instance: T) {
        self.collector.insert(&mut self.context, instance);
    }
}
