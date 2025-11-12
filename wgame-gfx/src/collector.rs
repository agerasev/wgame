use anyhow::Result;
use hashbrown::{HashMap, hash_map::Entry};

use crate::{Context, Instance, Resource, instance::AnyResource};

pub struct Collector<R: Resource = Box<dyn AnyResource>> {
    render_passes: HashMap<R, R::Storage>,
}

impl<R: Resource> Default for Collector<R> {
    fn default() -> Self {
        Self {
            render_passes: Default::default(),
        }
    }
}

impl Collector {
    pub fn push_any<T: Instance>(&mut self, ctx: &Context, instance: T) {
        let resource: Box<dyn AnyResource> = Box::new(instance.resource());
        let instances = self.get_or_init_storage(resource);
        instance.store(
            ctx,
            instances
                .downcast_mut()
                .expect("Error downcasting storage during push"),
        );
    }
}

impl<R: Resource> Collector<R> {
    pub fn push<T: Instance<Resource = R>>(&mut self, ctx: &Context, instance: T) {
        let resource = instance.resource();
        let instances = self.get_or_init_storage(resource);
        instance.store(ctx, instances);
    }

    fn get_or_init_storage(&mut self, resource: R) -> &mut R::Storage {
        match self.render_passes.entry(resource) {
            Entry::Vacant(entry) => {
                let storage = entry.key().new_storage();
                entry.insert(storage)
            }
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }

    pub fn renderers(&self) -> impl ExactSizeIterator<Item = Result<R::Renderer>> {
        self.render_passes.iter().map(|(k, v)| k.make_renderer(v))
    }
}
