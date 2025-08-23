use alloc::{
    boxed::Box,
    collections::btree_map::{BTreeMap, Entry},
};

use anyhow::Result;

use crate::{Context, Instance, Resources, instance::AnyResources};

pub struct Collector<R: Resources = Box<dyn AnyResources>> {
    render_passes: BTreeMap<R, R::Storage>,
}

impl<R: Resources> Default for Collector<R> {
    fn default() -> Self {
        Self {
            render_passes: Default::default(),
        }
    }
}

impl Collector {
    pub fn push_any<T: Instance>(&mut self, ctx: &Context, instance: T) {
        let resource: Box<dyn AnyResources> = Box::new(instance.get_resources());
        let instances = self.get_or_init_storage(resource);
        instance.store(
            ctx,
            instances
                .downcast_mut()
                .expect("Error downcasting storage during push"),
        );
    }
}

impl<R: Resources> Collector<R> {
    pub fn push<T: Instance<Resources = R>>(&mut self, ctx: &Context, instance: T) {
        let resource = instance.get_resources();
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

    pub fn renderers(&self) -> impl Iterator<Item = Result<R::Renderer>> {
        self.render_passes.iter().map(|(k, v)| k.make_renderer(v))
    }
}
