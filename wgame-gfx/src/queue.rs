use alloc::boxed::Box;

use hashbrown::{HashMap, hash_map::Entry};

use crate::{Context, Instance, Renderer, renderer::AnyRenderer};

pub struct RenderQueue<R: Renderer + ?Sized = dyn AnyRenderer> {
    render_passes: HashMap<Box<R>, R::Storage>,
}

impl<R: Renderer + ?Sized> Default for RenderQueue<R> {
    fn default() -> Self {
        Self {
            render_passes: Default::default(),
        }
    }
}

impl RenderQueue {
    pub fn push<T: Instance>(&mut self, ctx: impl Context, instance: T) {
        let renderer: Box<dyn AnyRenderer> = Box::new(instance.get_renderer());
        let instances = match self.render_passes.entry(renderer) {
            Entry::Vacant(entry) => {
                let storage = entry.key().new_storage();
                entry.insert(storage)
            }
            Entry::Occupied(entry) => entry.into_mut(),
        };
        instance.store(
            ctx,
            instances.downcast_mut().expect("Error downcasting storage"),
        );
    }
}

impl<R: Renderer + ?Sized> RenderQueue<R> {
    pub fn iter(&self) -> impl Iterator<Item = (&R, &R::Storage)> {
        self.render_passes.iter().map(|(k, v)| (k.as_ref(), v))
    }
}
