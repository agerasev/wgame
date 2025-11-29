use std::{any::Any, rc::Rc};

use hashbrown::{HashMap, hash_map::EntryRef};

use crate::{Camera, Context, Instance, Renderer, Resource, resource::AnyResource};

#[derive(Default)]
pub struct Collector {
    items: HashMap<Rc<dyn AnyResource>, Box<dyn Any>>,
}

impl Collector {
    pub fn insert_with_context<T: Instance>(&mut self, ctx: &T::Context, instance: T) {
        let resource = instance.resource();
        let storage = self.get_or_init_storage(resource);
        instance.store(ctx, storage);
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

impl<C: Context> Renderer<C> for Collector {
    fn insert<T: Instance<Context = C>>(&mut self, instance: T) {
        self.insert_with_context(&C::default(), instance);
    }
}

pub struct CollectorWithContext<'a, C: Context = Camera> {
    pub collector: &'a mut Collector,
    pub context: C,
}

impl<C: Context> Renderer<C> for CollectorWithContext<'_, C> {
    fn insert<T: Instance<Context = C>>(&mut self, instance: T) {
        self.collector.insert_with_context(&self.context, instance);
    }
}
