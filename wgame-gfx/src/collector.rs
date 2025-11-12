use std::{any::Any, rc::Rc};

use anyhow::Result;
use hashbrown::{HashMap, hash_map::EntryRef};

use crate::{
    Camera, Instance, Object, Renderer, Resource,
    resource::AnyResource,
    types::{Color, Transform},
};

#[derive(Default)]
pub struct Collector {
    render_passes: HashMap<Rc<dyn AnyResource>, Box<dyn Any>>,
}

impl Collector {
    pub fn push_instance<T: Instance>(&mut self, camera: &Camera, instance: T) {
        let resource = instance.resource();
        let instances = self.get_or_init_storage(resource);
        instance.store(camera, instances);
    }

    pub fn push<T: Object>(&mut self, camera: &Camera, object: T) {
        object.collect_into(camera, self);
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

pub struct CollectorWithCamera<'a> {
    pub collector: &'a mut Collector,
    pub camera: Camera,
}

impl<'a> CollectorWithCamera<'a> {
    pub fn add<T: Object>(&mut self, object: T) {
        self.collector.push(&self.camera, object);
    }

    pub fn transform<'b: 'a>(&'b mut self, xform: impl Transform) -> CollectorWithCamera<'b> {
        Self {
            collector: &mut self.collector,
            camera: self.camera.transform(xform),
        }
    }
    pub fn color<'b: 'a>(&'b mut self, color: impl Color) -> CollectorWithCamera<'b> {
        Self {
            collector: &mut self.collector,
            camera: self.camera.color(color),
        }
    }
}
