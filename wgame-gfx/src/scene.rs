use std::{any::Any, cmp::Ordering, rc::Rc};

use hashbrown::{HashMap, hash_map::EntryRef};

use crate::{AnyResource, AnyStorage, Context, Instance, InstanceVisitor, Object, Resource};

pub struct Scene<C: Context> {
    items: HashMap<Rc<dyn AnyResource>, Box<dyn AnyStorage<C>>>,
}

impl<C: Context> Default for Scene<C> {
    fn default() -> Self {
        Self {
            items: HashMap::default(),
        }
    }
}

impl<C: Context> Scene<C> {
    fn add_instance<T: Instance<Context = C>>(&mut self, instance: &T) {
        let resource = instance.resource();
        let any_storage = match self.items.entry_ref(&resource as &dyn AnyResource) {
            EntryRef::Vacant(entry) => entry.insert(Box::new(instance.new_storage())),
            EntryRef::Occupied(entry) => entry.into_mut(),
        };
        let storage = (any_storage.as_mut() as &mut dyn Any)
            .downcast_mut::<T::Storage>()
            .expect("Error downcasting storage during push");
        instance.store(storage);
    }

    pub fn add<T: Object<Context = C>>(&mut self, object: &T) {
        object.for_each_instance(self);
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn iter(&self) -> impl Iterator<Item = &dyn AnyStorage<C>> {
        let mut items: Vec<_> = self.items.iter().collect();
        items.sort_by(|a, b| {
            let (mut a, mut b) = (a.0.order(), b.0.order());
            loop {
                let (a, b) = (a.next(), b.next());
                if a.is_none() && b.is_none() {
                    break Ordering::Equal;
                }
                let (a, b) = (a.unwrap_or(0), b.unwrap_or(0));
                match a.cmp(&b) {
                    Ordering::Equal => (),
                    ord => break ord,
                }
            }
        });
        items.into_iter().map(|(_, s)| s.as_ref())
    }
}

impl<C: Context> InstanceVisitor<C> for Scene<C> {
    fn visit<T: Instance<Context = C>>(&mut self, instance: &T) {
        self.add_instance(instance);
    }
}
