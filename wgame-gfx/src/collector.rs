use std::{any::Any, rc::Rc};

use hashbrown::{HashMap, hash_map::EntryRef};

use crate::{
    Camera, Instance, Object, Resource,
    object::InstanceVisitor,
    resource::AnyResource,
    types::{Color, Transform},
};

#[derive(Default)]
pub struct Collector {
    items: HashMap<Rc<dyn AnyResource>, Box<dyn Any>>,
}

impl Collector {
    pub fn push<T: Instance>(&mut self, camera: &Camera, instance: T) {
        let resource = instance.resource();
        let instances = self.get_or_init_storage(resource);
        instance.store(camera, instances);
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

impl InstanceVisitor for Collector {
    fn visit<T: Instance>(&mut self, camera: &Camera, instance: T) {
        self.push(camera, instance);
    }
}

pub struct CollectorWithCamera<'a> {
    pub collector: &'a mut Collector,
    pub camera: Camera,
}

impl<'a> CollectorWithCamera<'a> {
    pub fn add<T: Object>(&mut self, object: T) {
        object.visit_instances(&self.camera, self.collector);
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
