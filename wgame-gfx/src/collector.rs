use std::{any::Any, rc::Rc};

use anyhow::Result;
use hashbrown::{HashMap, hash_map::EntryRef};

use crate::{Context, Instance, Renderer, Resource, resource::AnyResource};

#[derive(Default)]
pub struct Collector {
    render_passes: HashMap<Rc<dyn AnyResource>, Box<dyn Any>>,
}

impl Collector {
    pub fn push<T: Instance>(&mut self, ctx: &Context, instance: &T) {
        let resource = instance.resource();
        let instances = self.get_or_init_storage(resource);
        instance.store(ctx, instances);
    }

    fn get_or_init_storage<R: Resource>(&mut self, resource: R) -> &mut R::Storage {
        match self.render_passes.entry_ref(resource.as_any()) {
            EntryRef::Vacant(entry) => entry.insert(resource.as_any().new_dyn_storage()),
            EntryRef::Occupied(entry) => entry.into_mut(),
        }
        .downcast_mut()
        .expect("Error downcasting storage during push")
    }

    pub fn renderers(&self) -> impl ExactSizeIterator<Item = Result<Box<dyn Renderer>>> {
        self.render_passes.iter().map(|(k, v)| k.make_renderer(v))
    }
}
